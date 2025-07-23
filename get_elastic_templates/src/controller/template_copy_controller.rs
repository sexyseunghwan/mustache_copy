use crate::common::*;

//use crate::traits::es_query_service::*;

//use crate::repository::es_repository_impl::*;

use crate::service::template_copy_service::*;
use crate::traits::es_repository::*;

#[derive(Debug, new)]
pub struct TemplateCopyController<R1: EsRepository, R2: EsRepository> {
    service: TemplateCopyService<R1, R2>,
}

impl<R1: EsRepository, R2: EsRepository> TemplateCopyController<R1, R2> {
    
    #[doc = "메인 컨트롤러 함수"]
    pub async fn handle_copy(&self) -> anyhow::Result<()> {        
        self.service.process_copy_mustache().await
    }
}
