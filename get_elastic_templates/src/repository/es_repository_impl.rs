use crate::common::*;

use crate::traits::es_repository::*;

use crate::utils_modules::io_utils::*;

use crate::model::{
    cluster_info::*,
    script_content::*
};

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsRepositoryImpl {
    pub cluster_name: String,
    pub es_clients: Vec<Arc<EsClient>>,
}

#[derive(Debug, Getters, Clone, new)]
pub struct EsClient {
    host: String,
    es_conn: Elasticsearch,
}

impl EsRepositoryImpl {
    #[doc = "Elasticsearch connection 인스턴스를 초기화 해주는 함수"]
    /// # Arguments
    /// * `path`    - Elasticsearch connection 정보가 존재하는 경로
    ///
    /// # Returns
    /// * anyhow::Result<Self>
    pub fn new(path: &str) -> Self {
        let copy_es_info: ClusterInfo =
            read_toml_from_file::<ClusterInfo>(path).unwrap_or_else(|e| {
                error!("[ERROR][EsRepositoryImpl->new] {:?}", e);
                panic!("[ERROR][EsRepositoryImpl->new] {:?}", e);
            });

        let mut es_clients: Vec<Arc<EsClient>> = Vec::new();

        for url in copy_es_info.hosts() {
            let parse_url: String = if copy_es_info.es_id() == "" && copy_es_info.es_pw() == "" {
                format!("http://{}", url)
            } else {
                format!(
                    "http://{}:{}@{}",
                    copy_es_info.es_id(),
                    copy_es_info.es_pw(),
                    url
                )
            };

            let es_url: Url = Url::parse(&parse_url).unwrap_or_else(|e| {
                error!("[ERROR][EsRepositoryImpl->new] {:?}", e);
                panic!("[ERROR][EsRepositoryImpl->new] {:?}", e);
            });

            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: Transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5, 0))
                .build()
                .unwrap_or_else(|e| {
                    error!("[ERROR][EsRepositoryImpl->new] {:?}", e);
                    panic!("[ERROR][EsRepositoryImpl->new] {:?}", e);
                });

            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: Arc<EsClient> = Arc::new(EsClient::new(url.to_string(), elastic_conn));
            es_clients.push(es_client);
        }

        EsRepositoryImpl {
            cluster_name: copy_es_info.cluster_name().to_string(),
            es_clients,
        }
    }

    #[doc = "특정 노드의 부하를 줄이기 위해 request를 각 노드로 분산시켜주는 함수"]
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(Elasticsearch) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error: Option<anyhow::Error> = None;
    
        let mut rng: StdRng = StdRng::from_entropy(); /* 랜덤 시드로 생성 */ 
        
        /* 클라이언트 목록을 셔플 -> StdRng를 사용하여 셔플*/ 
        let mut shuffled_clients: Vec<Arc<EsClient>>= self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);
        
        /* 셔플된 클라이언트들에 대해 순차적으로 operation 수행 */ 
        for es_client in shuffled_clients {
            
            let es_conn: Elasticsearch = es_client.es_conn.clone();

            match operation(es_conn).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }
        
        /* 모든 노드에서 실패했을 경우 에러 반환 */ 
        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }


}

#[async_trait]
impl EsRepository for EsRepositoryImpl {

    async fn get_mustache_template_infos(&self) -> anyhow::Result<Value> {

        let response: Response = self.execute_on_any_node(|es_client| async move {

            let response: Response = 
                es_client
                .cluster()
                .state(ClusterStateParts::Metric(&["metadata"]))
                .filter_path(&["metadata.stored_scripts"])
                .send()
                .await?;
            
            Ok(response)

        })
        .await?;

        if response.status_code().is_success() {
            let response_body: Value = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            Err(anyhow!("[ERROR][EsRepositoryImpl->get_mustache_template_infos()]"))
        }
    }


    async fn get_mustache_script(&self, template_name: &str) -> anyhow::Result<ScriptContent> {

        let endpoint: String = format!("/_scripts/{}", template_name);

        let response: Response = self.execute_on_any_node(move |es_client| 
            {
                let endpoint: String = endpoint.clone();

                async move {
                    let response: Response = 
                        es_client
                        .transport()
                        .send(
                            Method::Get,
                            endpoint.as_str(),
                            HeaderMap::new(),
                            None::<&str>,
                            None::<&[u8]>,
                            None::<Duration>,
                        )
                        .await?;

                    Ok(response)
                }
            }
        ).await?;

        if response.status_code().is_success() { 
            let value: serde_json::Value = response.json().await?;
            
            if let Some(script) = value.get("script") {
                let content: ScriptContent = serde_json::from_value(script.clone())?;
                Ok(content)
            } else {
                Err(anyhow!("[ERROR][EsRepositoryImpl->get_mustache_script] script not found"))
            }

        } else {
            Err(anyhow!("[ERROR][EsRepositoryImpl->get_mustache_script]"))
        }
    }


    async fn post_mustache_template(&self, template_name: &str, script_content: ScriptContent) -> anyhow::Result<()> {

        let endpoint: String = format!("/_scripts/{}", template_name);

        let body = json!({
            "script": {
                "lang": script_content.lang(),
                "source": script_content.source()
            }
        });
        
    
        let response: Response = self.execute_on_any_node(move |es_client| {
            
            let endpoint: String = endpoint.clone();
            let body: Value = body.clone();

            async move {
                let response: Response = es_client
                    .transport()
                    .send(
                        Method::Put,
                        endpoint.as_str(),
                        HeaderMap::new(),
                        None::<&str>,
                        Some(JsonBody::new(body)),
                        None::<Duration>,
                    )
                    .await?;

                Ok(response)
            }
        }).await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let text: String = response.text().await.unwrap_or_default();
            Err(anyhow!(
                "[ERROR][EsRepositoryImpl->put_mustache_script] Failed to upload script: {}",
                text
            ))
        }
    }
}
