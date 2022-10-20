use crate::authentication::MinecraftAuthenticator;

#[tokio::test]
async fn test() {
	let authenticator = MinecraftAuthenticator::new("1d644380-5a23-4a84-89c3-5d29615fbac2");

	let account = authenticator.initial_auth(|_, _, message| {
		println!("{}", message)
	}).await.unwrap();

	println!("{}", serde_json::to_string(&account).unwrap());

	let account = authenticator.refresh(account).await.unwrap();

	println!("{}", serde_json::to_string(&account).unwrap());
}