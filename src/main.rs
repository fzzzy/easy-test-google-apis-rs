extern crate diesel;
extern crate google_spanner1 as spanner1;
extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;

use diesel::r2d2;
use oauth2::ServiceAccountAccess;
use spanner1::CreateSessionRequest;
use spanner1::Error;
use spanner1::ExecuteSqlRequest;
use spanner1::Session;
use spanner1::Spanner;

use yup_oauth2::service_account_key_from_file;

use futures::future;
use futures::future::lazy;
use futures::future::Future;
use tokio_threadpool::ThreadPool;

const DATABASE_NAME: &'static str =
    "projects/sync-spanner-dev-225401/instances/spanner-test/databases/sync";

pub struct SpannerConnectionManager;

pub struct SpannerSession {
    hub: Spanner<hyper::Client, ServiceAccountAccess<hyper::Client>>,
    session: Session,
}

impl r2d2::ManageConnection for SpannerConnectionManager {
    type Connection = SpannerSession;
    type Error = Error;

    fn connect(&self) -> Result<Self::Connection, Error> {
        let secret = service_account_key_from_file(&String::from("service-account.json")).unwrap();
        let client = hyper::Client::with_connector(hyper::net::HttpsConnector::new(
            hyper_rustls::TlsClient::new(),
        ));
        let mut access = ServiceAccountAccess::new(secret, client);
        use yup_oauth2::GetToken;
        let _token = access
            .token(&vec!["https://www.googleapis.com/auth/spanner.data"])
            .unwrap();
        // println!("{:?}", token);
        let client2 = hyper::Client::with_connector(hyper::net::HttpsConnector::new(
            hyper_rustls::TlsClient::new(),
        ));
        let hub = Spanner::new(client2, access);
        let req = CreateSessionRequest::default();
        let session = hub
            .projects()
            .instances_databases_sessions_create(req, DATABASE_NAME)
            .doit()
            .unwrap()
            .1;
        Ok(SpannerSession { hub, session })
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Error> {
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

fn do_a_blocking_thing() -> Box<Future<Item = usize, Error = ()> + Send> {
    let m = SpannerConnectionManager {};
    let pool = r2d2::Pool::builder().build(m).unwrap();
    let spanner = pool.get().unwrap();
    let mut sql = ExecuteSqlRequest::default();
    sql.sql = Some("select count(*) from user_collections;".to_string());
    let session = spanner.session.name.as_ref().unwrap();
    let result = spanner
        .hub
        .projects()
        .instances_databases_sessions_execute_sql(sql, session)
        .doit();

    let rv = match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => {
                println!("{}", e);
                Box::new(future::err(()))
            }
        },
        Ok(res) => {
            println!("{:?}", res.1);
            Box::new(future::ok(42))
        }
    };
    rv
}

fn main() {
    let pool = ThreadPool::new();
    let fut = pool.spawn_handle(lazy(move || {
        println!("hello from another thread");
        do_a_blocking_thing()
    }));
    println!("hello world");
    println!("future result {}", fut.wait().unwrap());
}
