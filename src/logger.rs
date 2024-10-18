use actix_web::{ HttpRequest, http::StatusCode };
use tracing::{ warn, metadata::LevelFilter, Subscriber };
use tracing_subscriber::{ fmt, Layer, Registry, prelude::__tracing_subscriber_SubscriberExt };
use serde_json::Value;

pub struct Logger {}
impl Logger {
    pub fn new() -> Logger {
        Logger {}
    }
	pub fn record(&self, req: HttpRequest, body: String, status_code: StatusCode) {
	    let binding = req.connection_info();
	    let src_ip = binding.realip_remote_addr().unwrap_or_default();

	    let path = req.path();
	    let qs = req.query_string();

	    // Collect headers as a list of (name, value) pairs
	    let headers: Vec<(String, String)> = req.headers()
	        .iter()
	        .map(|(name, value)| {
	            let header_name = name.as_str().to_string();
	            let header_value = value.to_str().unwrap_or_default().to_string();
	            (header_name, header_value)
	        })
	        .collect();

	    // Extract the host header and split it into host and dest_port
	    let host_header = headers.iter().find(|(name, _)| name == "host").map(|(_, value)| value.clone()).unwrap_or_default();
	    let (host, dest_port) = if let Some((host, port)) = host_header.split_once(':') {
	        (host.to_string(), port.to_string())  // Split host and port
	    } else {
	        (host_header.clone(), "80".to_string())  // Default port if not provided
	    };

	    // Extract the User-Agent header
	    let user_agent = headers.iter().find(|(name, _)| name == "user-agent").map(|(_, value)| value.clone()).unwrap_or_default();

	    // Split the User-Agent string by common delimiters
	    let user_agent_parts: Vec<&str> = user_agent.split(|c| c == '(' || c == ')' || c == ';').collect();

	    // Extract common fields from User-Agent
	    let browser = user_agent_parts.get(0).unwrap_or(&"").trim();
	    let platform = user_agent_parts.get(1).unwrap_or(&"").trim();
	    let device_info = user_agent_parts.get(2).unwrap_or(&"").trim();
	    let engine_version = user_agent_parts.get(3).unwrap_or(&"").trim();

	    // Try to parse the body as JSON and flatten it if possible
	    let flattened_body = if let Ok(json_body) = serde_json::from_str::<Value>(&body) {
	        json_body.as_object()
	            .map(|obj| {
	                obj.iter()
	                    .map(|(k, v)| format!("{}:{}", k, v.as_str().unwrap_or("")))
	                    .collect::<Vec<String>>()
	                    .join(",")
	            })
	            .unwrap_or(body)  // Fallback to original body if it's not a JSON object
	    } else {
	        body  // If the body isn't JSON, keep it as is
	    };

	    // Log fields, omitting empty fields
		// Headers kept failing with invalid JSON, adding fields manually based on splits / extractions
	    warn!(
	        target: "honeyaml::access-log",
	        src_ip = src_ip,
	        path = path,
	        method = req.method().to_string(),
	        query_string = if !qs.is_empty() { Some(qs) } else { None },
	        // Log the flattened body as a single string
	        body = if !flattened_body.is_empty() { Some(flattened_body.as_str()) } else { None },
	        status_code = status_code.as_u16(),
	        host = host,
	        dest_port = dest_port,
	        user_agent_browser = if !browser.is_empty() { Some(browser) } else { None },
	        user_agent_platform = if !platform.is_empty() { Some(platform) } else { None },
	        user_agent_device_info = if !device_info.is_empty() { Some(device_info) } else { None },
	        user_agent_engine_version = if !engine_version.is_empty() { Some(engine_version) } else { None },
	        authorization = headers.iter().find(|(name, _)| name == "authorization").map(|(_, value)| if !value.is_empty() { Some(value.as_str()) } else { None }).flatten(),
	        accept = headers.iter().find(|(name, _)| name == "accept").map(|(_, value)| if !value.is_empty() { Some(value.as_str()) } else { None }).flatten(),
	        content_type = headers.iter().find(|(name, _)| name == "content-type").map(|(_, value)| if !value.is_empty() { Some(value.as_str()) } else { None }).flatten(),
	        content_length = headers.iter().find(|(name, _)| name == "content-length").map(|(_, value)| if !value.is_empty() { Some(value.as_str()) } else { None }).flatten(),
	        user_agent = if !user_agent.is_empty() { Some(user_agent.as_str()) } else { None }
	    );
	}
}

pub fn setup_logger(
    directory: String,
    file_name_prefix: String,
    log_severity: LevelFilter
) -> std::io::Result<impl Subscriber> {
    std::fs::create_dir_all(directory.clone())?;
    let file_appender = tracing_appender::rolling::never(directory, file_name_prefix); // Replace daily with never for T-Pot (flat log filename)
    // enable non_blocking if necessary
    // let (non_blocking, _guard1) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::Layer::new().json().flatten_event(true).with_writer(file_appender).with_filter(log_severity); // Add flatten_event(true) for T-Pot

    let stdout_log = tracing_subscriber::fmt::layer().with_filter(log_severity);
    let subscriber = Registry::default().with(stdout_log).with(file_layer);
    Ok(subscriber)
}

pub fn verbosity_to_level_filter(severity: u8) -> LevelFilter {
    match severity {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        2 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use glob::glob;
    #[test]
    fn test_logging() {
        let level = verbosity_to_level_filter(10);
        assert!(level == LevelFilter::TRACE);
        let level = verbosity_to_level_filter(0);
        assert!(level == LevelFilter::WARN);
        let level = verbosity_to_level_filter(1);
        assert!(level == LevelFilter::INFO);
        let level = verbosity_to_level_filter(2);
        assert!(level == LevelFilter::DEBUG);

        let req = actix_web::test::TestRequest
            ::default()
            .insert_header(actix_web::http::header::ContentType::plaintext())
            .to_http_request();
        let body = "F00oo00oo".to_string();
        let status_code = actix_web::http::StatusCode::OK;

        let sub = setup_logger("/tmp".to_string(), "honeyaml.log".to_string(), level).unwrap();

        tracing::subscriber::set_global_default(sub).unwrap();

        let l = Logger::new();
        l.record(req, body.clone(), status_code);

        if let Ok(paths) = glob("/tmp/honeyaml.log*") {
            for path in paths.flatten() {
                let s = std::fs::read_to_string(path.clone()).unwrap();
                assert!(s.contains(&body));
                _ = std::fs::remove_file(path);
            }
        } else {
            assert_eq!("cannot find matching pattern", "doesnt matter")
        }
    }
}
