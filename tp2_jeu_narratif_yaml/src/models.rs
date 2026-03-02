use serde::Deserialize;
use std::collections::HashSet;


#[derive(Deserialize, Debug)]
pub struct Scenario {
    pub start_scene: String,
    pub initial_hp: i32,
    pub scenes: Vec<Scene>,
}


#[derive(Deserialize, Debug)]
pub struct Scene {
    pub id: String,
    pub title: String,
    pub text: String,
    pub choices: Option<Vec<Choices>>,
    pub found_item : Option<String>,
    pub ending: Option<String>,
    pub hp_delta: Option<i8>,
}

#[derive(Deserialize, Debug)]
pub struct Choices {
    pub label: String,
    pub next: String,
    pub required_item: Option<String>
}

pub struct GameState {
    pub current_scene_id: String,
    pub inventory: Vec<String>,
    pub hp: i32,
    pub is_running: bool,
}


pub fn validate_scenario(scenario: &Scenario) -> Result<(), String> {
    let mut scene_ids = HashSet::new();
    for scene in &scenario.scenes {
        if !scene_ids.insert(&scene.id) {
            return Err(format!("ID de scène en double détecté : {}", scene.id));
        }
    }

    if !scene_ids.contains(&scenario.start_scene) {
        return Err(format!(
            "La scène de départ '{}' est introuvable.",
            scenario.start_scene
        ));
    }

    for scene in &scenario.scenes {
        if let Some(ref choices) = scene.choices {
            for (index, choice) in choices.iter().enumerate() {
                if !scene_ids.contains(&choice.next) {
                    return Err(format!(
                        "Erreur dans la scène '{}' : le choix {} ('{}') pointe vers '{}', qui n'existe pas.",
                        scene.id, index, choice.label, choice.next
                    ));
                }
            }
        }
    }

    Ok(())
}
