use crate::common::*;

use crate::model::script_content::*;

#[async_trait]
pub trait EsRepository: Send + Sync {
    
    #[doc = "특정 elasticsearch cluster 에서 mustache 정보를 가져와주는 함수"]
    async fn get_mustache_template_infos(&self) -> anyhow::Result<Value>;
    
    #[doc = "특정 mustache template 을 가져와주는 함수"]
    async fn get_mustache_script(&self, template_name: &str) -> anyhow::Result<ScriptContent>;

    #[doc = "mustache template 을 복사해주는 함수"]
    async fn post_mustache_template(&self, template_name: &str, script_content: ScriptContent) -> anyhow::Result<()>;
    
}