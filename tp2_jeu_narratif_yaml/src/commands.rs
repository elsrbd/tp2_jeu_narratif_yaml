use crate::models::{Scenario, GameState};
use crate::errors::GameError;

pub enum CommandOutcome {
    DisplayOnly,
    Moved,
    GameOver(String),
}

pub trait GameCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError>;
}

pub struct LookCommand;
pub struct ChooseCommand { pub choice_index: usize }
pub struct InventoryCommand;
pub struct StatusCommand;
pub struct QuitCommand;

impl GameCommand for LookCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
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
        Ok(CommandOutcome::DisplayOnly)
    }
}


impl GameCommand for ChooseCommand {
    fn execute(&self, scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        let current_scene = scenario.scenes.iter()
            .find(|s| s.id == state.current_scene_id)
            .ok_or(GameError::SceneNotFound(state.current_scene_id.clone()))?;

        let choices = current_scene.choices.as_ref().ok_or(GameError::InvalidChoice)?;
        let choice = choices.get(self.choice_index).ok_or(GameError::InvalidChoice)?;

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
            if !state.inventory.contains(item) {
                state.inventory.push(item.clone());
            }
        }

        state.hp += next_scene.hp_delta.unwrap_or(0) as i32;

        if state.hp <= 0 {
            state.is_running = false;
            return Ok(CommandOutcome::GameOver("Blessures".to_string()));
        }

        if let Some(ref end_type) = next_scene.ending {
            state.is_running = false;
            return Ok(CommandOutcome::GameOver(format!("Fin: {}", end_type)));
        }

        Ok(CommandOutcome::Moved)
    }
}

impl GameCommand for InventoryCommand {
    fn execute(&self, _scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        if state.inventory.is_empty() {
            println!("Votre inventaire est vide.");
        } else {
            println!("Inventaire : {:?}", state.inventory);
        }
        Ok(CommandOutcome::DisplayOnly)
    }
}

impl GameCommand for StatusCommand {
    fn execute(&self, _scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        println!("Points de Vie : {}", state.hp);
        println!("Lieu actuel : {}", state.current_scene_id);
        Ok(CommandOutcome::DisplayOnly)
    }
}

impl GameCommand for QuitCommand {
    fn execute(&self, _scenario: &Scenario, state: &mut GameState) -> Result<CommandOutcome, GameError> {
        state.is_running = false;
        println!("Merci d'avoir joué !");
        Ok(CommandOutcome::DisplayOnly)
    }
}


pub fn parse_command(line: &str) -> Result<Box<dyn GameCommand>, String> {
    let tokens: Vec<&str> = line.trim().split_whitespace().collect();

    match tokens.as_slice() {
        ["look"] => Ok(Box::new(LookCommand)),
        ["inventory"] => Ok(Box::new(InventoryCommand)),
        ["status"] => Ok(Box::new(StatusCommand)),
        ["quit"] => Ok(Box::new(QuitCommand)),
        ["choose", n] => {
            let idx = n.parse::<usize>().map_err(|_| "L'index doit être un nombre")?;
            Ok(Box::new(ChooseCommand { choice_index: idx }))
        },
        _ => Err("Commande inconnue.".to_string()),
    }
}