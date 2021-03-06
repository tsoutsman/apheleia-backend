lazy_static::lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonResponse {
    student_id: String,
    // The response contains more fields, but the student_id is the only one we
    // care about. The struct will still deserialize successfully despite the additional
    // fields in the response.
}

#[inline]
async fn sbhs_token_to_id(token: String) -> Result<u32, Box<dyn std::error::Error>> {
    const API_ENDPOINT: &str = "https://student.sbhs.net.au/api/details/userinfo";

    let id = HTTP_CLIENT
        .get(API_ENDPOINT)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await?
        .json::<JsonResponse>()
        .await?
        .student_id
        .parse()?;

    Ok(id)
}

#[inline]
async fn no_auth(token: String) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(token.parse()?)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> apheleia::Result<()> {
    for arg in std::env::args() {
        if arg == "--no-auth" {
            return apheleia::serve(
                no_auth,
                apheleia::Root(0),
            ).await;
        }
    }
    apheleia::serve(sbhs_token_to_id, apheleia::Root(0)).await
}
