use super::*;
pub fn get(url: &str) -> Result<BlockingResponse, HttpError> {
    ReqwestBlockingHttpClient.get(url)
}
