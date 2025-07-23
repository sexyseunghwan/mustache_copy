/*
Author      : Seunghwan Shin 
Create date : 2024-10-04 
Description : Elasticsearch cluster 에 존재하는 mustache 템플릿 리스트를 뽑아주는 기능.
    
History     : 2024-10-04 Seunghwan Shin       # [v.1.0.0] first create.
              2024-10-08 Seunghwan Shin       # [v.1.1.0] 추상화 아키텍쳐로 변경.
              2025-07-23 Seunghwan Shin       # [v.2.0.0] 특정 클러스터에서 mustache template을 뽑아서 다른 클러스터에 템플릿을 이전해주는 기능으로 변경.
*/ 
mod common;
use common::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod controller;
use controller::template_copy_controller::*;

mod repository;
use repository::es_repository_impl::*;

mod traits;

mod service;
use service::template_copy_service::*;

mod model;

#[tokio::main]
async fn main() {
    dotenv().ok();
    set_global_logger();
    info!("Start the template finder program");

    let from_es_info: String = env::var("FROM_ES_INFO_PATH").unwrap_or_else(|e| {
        error!("[ERROR][main] {:?}", e);
        panic!("[ERROR][main] {:?}", e);
    });

    let to_es_info: String = env::var("TO_ES_INFO_PATH").unwrap_or_else(|e| {
        error!("[ERROR][main] {:?}", e);
        panic!("[ERROR][main] {:?}", e);
    });

    /* ============ 의존주입 과정 ============ */
    let copy_es_conn: EsRepositoryImpl = EsRepositoryImpl::new(&from_es_info);
    let target_es_conn: EsRepositoryImpl = EsRepositoryImpl::new(&to_es_info);

    let template_copy_service: TemplateCopyService<EsRepositoryImpl, EsRepositoryImpl> =
        TemplateCopyService::new(copy_es_conn, target_es_conn);

    let controller: TemplateCopyController<EsRepositoryImpl, EsRepositoryImpl> = TemplateCopyController::new(template_copy_service);

    controller.handle_copy().await.unwrap_or_else(|e| {
        error!("[ERROR][main] {:?}", e);
        panic!("[ERROR][main] {:?}", e);
    });
}
