extern crate json;
extern crate curl;

use curl::easy::Easy;
use self::json::JsonValue;
use image::EncodableLayout;

pub struct GameModel {
    pub(crate) home_team: String,
    pub(crate) away_team: String,
    pub(crate) description: String,
    pub(crate) image: Vec<u8>,
    pub(crate) image_w: u32,
    pub(crate) image_h: u32
}

impl GameModel {
    pub fn from_json(data: &json::JsonValue) -> Self {
        let (home_team, away_team) = GameModel::teams_from_json(data);

        GameModel {
            home_team,
            away_team,
            description: GameModel::desc_from_json(data),
            image: GameModel::image_data_from_json(data),
            image_w: data["content"]["editorial"]["recap"]["mlb"]["image"]["cuts"][18]["width"].as_u32().unwrap_or(215),
            image_h: data["content"]["editorial"]["recap"]["mlb"]["image"]["cuts"][18]["height"].as_u32().unwrap_or(121)
        }
    }

    fn teams_from_json(data: &json::JsonValue) -> (String, String) {
        ( data["teams"]["home"]["team"]["name"].as_str().unwrap().to_owned(), data["teams"]["away"]["team"]["name"].as_str().unwrap().to_owned() )
    }

    fn desc_from_json(data: &json::JsonValue) -> String {
        if let Some(desc) = data["content"]["editorial"]["recap"]["mlb"]["headline"].as_str() {
            desc.to_owned()
        } else { String::new() }
    }

    fn image_data_from_json(data: &json::JsonValue) -> Vec<u8> {
        let mut result = Vec::new();
        match data["content"]["editorial"]["recap"]["mlb"]["image"]["cuts"][16]["src"].as_str() {
            None => { result }
            Some(url) => {
                let jpg_bytes = fetch_bytes(url.to_owned());
                match image::load_from_memory_with_format(jpg_bytes.as_bytes(), image::ImageFormat::Jpeg) {
                    Ok(loaded_image) => {
                        loaded_image.write_to(&mut result, image::ImageFormat::Png).unwrap();
                        result
                    }
                    Err(_) => { result }
                }
            }
        }
    }
}

pub fn fetch_bytes(url: String) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut easy_handle = Easy::new();
    easy_handle.url(url.as_str()).unwrap();

    {
        let mut transfer = easy_handle.transfer();
        transfer.write_function(|data| {
            bytes.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    bytes
}

pub fn fetch_json(url: String) -> JsonValue {
    let bytes = fetch_bytes(url);
    let json_string = String::from_utf8(bytes).unwrap();
    json::parse(json_string.as_str()).unwrap()
}

pub fn fetch_games(url: String) -> Vec<GameModel> {
    let data = fetch_json(url);
    let mut models= Vec::new();

    if let json::JsonValue::Array(games) = &data["dates"][0]["games"] {
        for game in games {
            if ! game.is_null() {
                models.push(GameModel::from_json(game))
            }
        }
    }

    models
}

pub fn make_url_for_date(_: String) -> String {
    // Todo -- Construct Url from date string
    "http://statsapi.mlb.com/api/v1/schedule?hydrate=game(content(editorial(recap))),decisions&date=2020-09-01&sportId=1".to_owned()
}