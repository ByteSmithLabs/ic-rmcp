use ic_cdk::management_canister::{
    http_request_with_closure, HttpMethod, HttpRequestArgs, HttpRequestResult,
};
use ic_cdk::{init, query, update};
use ic_http_certification::{HttpRequest, HttpResponse, StatusCode};
use ic_rmcp::{model::*, schema_for_type, Context, Error, Handler, Server};
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::cell::RefCell;
use candid::CandidType;

thread_local! {
    static ARGS : RefCell<InitArgs> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
struct WeatherUnits {
    time: String,
    interval: String,
    temperature: String,
    windspeed: String,
    winddirection: String,
    is_day: String,
    weathercode: String,
}

#[derive(Serialize, Deserialize)]
struct CurrentWeather {
    time: String,
    interval: f64,
    temperature: f64,
    windspeed: f64,
    winddirection: f64,
    is_day: f64,
    weathercode: f64,
}

#[derive(Serialize, Deserialize)]
struct WeatherResponse {
    latitude: f64,
    longitude: f64,
    generationtime_ms: f64,
    utc_offset_seconds: f64,
    timezone: String,
    timezone_abbreviation: String,
    elevation: f64,
    current_weather_units: WeatherUnits,
    current_weather: CurrentWeather,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct WeatherRequest {
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize, CandidType, Default)]
struct InitArgs {
    api_key: String,
    replicated: bool,
}

#[init]
fn init(config: InitArgs) {
    ARGS.with_borrow_mut(|args| *args = config);
}

async fn fetch_weather(latitude: f64, longitude: f64) -> Result<WeatherResponse, String> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
        latitude, longitude
    );

    let replicated = ARGS.with_borrow(|args| args.replicated);

    let body = http_request_with_closure(
        &HttpRequestArgs {
            is_replicated: Some(replicated),
            url,
            max_response_bytes: Some(10_000),
            method: HttpMethod::GET,
            headers: vec![],
            body: None,
            transform: None,
        },
        |raw| HttpRequestResult {
            status: raw.status.clone(),
            body: raw.body.clone(),
            headers: vec![],
        },
    )
    .await
    .map_err(|err| format!("HTTP request failed: {}", err))?
    .body;

    from_slice::<WeatherResponse>(&body)
        .map_err(|err| format!("Failed to parse weather data: {}", err))
}

#[query]
fn http_request(_: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .with_status_code(StatusCode::OK)
        .with_upgrade(true)
        .build()
}

struct Weather;

impl Handler for Weather {
    fn get_info(&self, _: Context) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "Weather".to_string(),
                version: "1.0.0".to_string(),
            },
            instructions: Some("This server provides weather information using the Open-Meteo API. You can get current weather conditions for any location by providing latitude and longitude coordinates. If no coordinates are provided, it defaults to Berlin, Germany.".to_string()),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _: Context,
        _: Option<PaginatedRequestParam>,
    ) -> Result<ListToolsResult, Error> {
        Ok(ListToolsResult {
            next_cursor: None,
            tools: vec![
                Tool::new(
                    "get_current_weather",
                    "Get current weather conditions for a location. Provide latitude and longitude, or it defaults to Berlin.",
                    schema_for_type::<WeatherRequest>(),
                ),
            ],
        })
    }

    async fn call_tool(
        &self,
        _: Context,
        requests: CallToolRequestParam,
    ) -> Result<CallToolResult, Error> {
        match requests.name.as_ref() {
            "get_current_weather" => {
                let params: WeatherRequest = if let Some(args) = requests.arguments {
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|_| {
                        Error::invalid_params("Invalid weather request parameters", None)
                    })?
                } else {
                    WeatherRequest {
                        latitude: None,
                        longitude: None,
                    }
                };

                let latitude = params.latitude.unwrap_or(52.52);
                let longitude = params.longitude.unwrap_or(13.41);

                match fetch_weather(latitude, longitude).await {
                    Ok(weather) => {
                        let weather_desc = match weather.current_weather.weathercode as i32 {
                            0 => "Clear sky",
                            1..=3 => "Partly cloudy",
                            45..=48 => "Foggy",
                            51..=67 => "Rainy",
                            71..=86 => "Snowy",
                            95..=99 => "Thunderstorm",
                            _ => "Unknown",
                        };

                        let is_day = if weather.current_weather.is_day == 1.0 {
                            "Day"
                        } else {
                            "Night"
                        };

                        let weather_info = format!(
                            "Weather for {}, {} (lat: {}, lon: {})\n\
                            Time: {}\n\
                            Temperature: {}°C\n\
                            Wind: {:.1} km/h from {:.0}°\n\
                            Conditions: {} ({})\n\
                            Timezone: {} ({})",
                            weather.timezone,
                            weather.timezone_abbreviation,
                            weather.latitude,
                            weather.longitude,
                            weather.current_weather.time,
                            weather.current_weather.temperature,
                            weather.current_weather.windspeed,
                            weather.current_weather.winddirection,
                            weather_desc,
                            is_day,
                            weather.timezone,
                            weather.timezone_abbreviation
                        );

                        Ok(CallToolResult::success(
                            Content::text(weather_info).into_contents(),
                        ))
                    }
                    Err(err) => Err(Error::internal_error(err, None)),
                }
            }
            _ => Err(Error::invalid_params("Tool not found", None)),
        }
    }
}

#[update]
async fn http_request_update(req: HttpRequest<'_>) -> HttpResponse<'_> {
    Weather {}
        .handle(&req, |headers| -> bool {
            headers
                .iter()
                .any(|(k, v)| k == "x-api-key" && *v == ARGS.with_borrow(|args| args.api_key.clone()))
        })
        .await
}

ic_cdk::export_candid!();
