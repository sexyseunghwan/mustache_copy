use crate::common::*;

use crate::traits::es_repository::*;

use crate::model::script_content::*;

#[derive(Debug, new)]
pub struct TemplateCopyService<R1: EsRepository, R2: EsRepository> {
    source_repo: R1,
    target_repo: R2,
}

impl<R1: EsRepository, R2: EsRepository> TemplateCopyService<R1, R2> {
    
    #[doc = "특정 클러스터의 mustache template 을 다른 클러스터로 복사해주는 서비스 함수"]
    pub async fn process_copy_mustache(&self) -> anyhow::Result<()> {

        /* 현재 존재하는 mustache template 정보들 */
        let source_template_infos: Value = self.source_repo.get_mustache_template_infos().await?;

        /* 각 mustache template 의 이름을 가져와주는 함수 */
        let template_name_list: Vec<String> = source_template_infos
            .get("metadata")
            .ok_or_else(|| anyhow!("[ERROR][TemplateCopyService->process_copy_mustache] The `metadata` field is missing."))?
            .get("stored_scripts")
            .ok_or_else(|| anyhow!("[ERROR][TemplateCopyService->process_copy_mustache] The `stored_scripts` field is missing."))?
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("[ERROR][TemplateCopyService->process_copy_mustache] stored_scripts is not an object."))?
            .keys()
            .cloned()
            .collect();
        
        
        for template in template_name_list {
            
            let script_name: String = template;
            let script_contents: ScriptContent = self.source_repo.get_mustache_script(&script_name).await?;

            match self.target_repo.post_mustache_template(&script_name, script_contents).await {
                Ok(_) => {
                    info!("[INFO][TemplateCopyService->process_copy_mustache] Copy {} template completed.", script_name);
                },
                Err(e) => {
                    error!("[ERROR][TemplateCopyService->process_copy_mustache] Failed to copy {} template. : {:?}", script_name, e);
                    continue;
                }
            }
        }

        Ok(())
    }

} 
