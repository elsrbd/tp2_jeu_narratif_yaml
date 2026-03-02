use serde::{Serialize, Deserialize};
use std::{collections::HashMap, thread::current};

#[derive(Serialize, Deserialize, Debug)]
pub struct Scenario {
    pub start_scene: String,
    pub initial_hp: i32,
    pub scenes: Vec<Scene>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Scene {
    pub id: String,
    pub title: String,
    pub text: String,
    pub choices: Option<Vec<Choices>>,
    pub found_item : Option<String>,
    pub ending: Option<String>,
    pub hp_delta: Option<i8>,
}

#[derive(Serialize, Deserialize)]
pub struct Choices {
    pub label: String,
    pub next: SceneID,
}

pub struct GameState {
    pub current_scene_id: String,
    pub inventory: Vec<String>,
    pub hp: i32,
    pub is_running: bool,
}

pub enum CommandOutCome {
    DisplayOnly,
    Moved,
    GameOver(String),
}

pub trait GameCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutCome, GameError>;
}

pub struct LookCommand;

pub struct ChooseCommand {
    pub choixe_index: usize,
}

impl GameCommand for LookCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutCome, GameError> {
        let scene = scenario.scenes.iter()
            .find(|s| s.id == state.current_scene_id)
            .ok_or(GameError::SceneNotFound(state.current_scene_id.clone()))?;

        println!("\n---- {} ----", scene.title);
        println!("{}", scene.text);

        if let Some(ref choices) = scene.choices {
            for (i, c) in choices.iter().enumerate() {
                println!("{}: {}", i, c.label);
            }
        }
        Ok(CommandOutCome::DisplayOnly)
    }
}

impl GameCommand for ChooseCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutCome, GameError> {
        let current_scene = scenario.scenes.iter()
            .find(|s| s.id == state.current_scene_id)
            .ok_or(GameError::SceneNotFound(state.current_scene_id.clone()))?;

        let choices = current_scene.choices.as_ref().ok_or(GameError::InvalidChoice)?;
        let choice: choices.get(self.choice_index).ok_or(GameError::InvalidChoice)?;

        if let Some(ref required) = choice.required_item {
            if !state.inventory.contains(required) {
                return Err(GameError::MissingItem(required.clone()));
    
            }
        }
        state.current_scene_id = choice.next.clone();
        let next_scene = scenario.scenes.iter()
            .find(|s| s.id == state.current_scene_id)
            .ok_or(GameError::SceneNotFound(state.current_scene_id.clone()))?;
        if let Some(ref item) = next_scene.found_item {
            if !state.inventory.countains(item) {
                state.inventory.push(item.clone());
            }
        }

        state.hp += next_scene.hp_delta.unwrap_or(0) as i32;

        if state.hp <= 0 {
            state.is_running = false;
            return Ok(CommandOutCome::GameOver("Blessures".to_string()));
        }

        if let Some(ref end_type) = next_scene.ending {
            state.is_running = false;
            return Ok(CommandOutCome::GameOver(format!("Fin: {}", end_type)));
        }

        Ok(CommandOutCome::Moved)
    }
}

fn main() {
    let yaml = "story.yaml";
    let scenes: Vec<Scene> = serde_yaml::from_str(yaml.to_string()).unwrap(); 
}
