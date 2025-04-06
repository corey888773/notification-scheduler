use mongodb::bson::doc;

use super::notifications::Notification;

#[derive(Clone)]
pub struct DbContext {
	pub db: mongodb::Database,
	pub notifications_collection: mongodb::Collection<Notification>,
}

impl DbContext {
	pub async fn new(url: &str) -> Result<Self, mongodb::error::Error> {
		let client_options = mongodb::options::ClientOptions::parse(url).await?;
		let client = mongodb::Client::with_options(client_options)?;

		let db = client.database("notifications");
		let notifications_collection = db.collection::<Notification>("notifications");

		create_notifications_indexes(&notifications_collection).await?;
		Ok(DbContext {
			db,
			notifications_collection,
		})
	}
}

async fn create_notifications_indexes(
	coll: &mongodb::Collection<Notification>,
) -> Result<(), mongodb::error::Error> {
	let priority_status_index = mongodb::IndexModel::builder()
		.keys(doc! { "priority": 1, "status": 1, "scheduledTime": 1 })
		.options(mongodb::options::IndexOptions::builder().build())
		.build();

	coll.create_index(priority_status_index).await?;
	Ok(())
}
