This is a demo of using google-apis-rs code inside a tokio-threadpool.

To use this, you need a Google Service Account Key json file. Create and
download the private key following the instructions [here](https://cloud.google.com/iam/docs/creating-managing-service-account-keys#creating_service_account_keys)
and save the downloaded json file in this directory as `service-account.json`.

Create a spanner database instance, edit `src/main.rs`, and change
DATABASE_INSTANCE to point to your spanner instance.

Run `cargo run`. It will print out the number of databases in your spanner
instance.
