use serde::{Serialize, Deserialize};
use std::{collections::HashMap, thread::current};
use std::io::{self, Write}; 
use std::collections::HashSet;

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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug)]
pub enum GameError {
    InvalidChoice,
    MissingItem(String),
    SceneNotFound(String),
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

fn main() {
    let yaml_content = std::fs::read_to_string("story.yaml")
        .expect("Erreur : Impossible de trouver le fichier story.yaml");

    let scenario: Scenario = serde_yaml::from_str(&yaml_content)
        .expect("Erreur : Le format du fichier YAML est invalide");

    if let Err(e) = validate_scenario(&scenario) {
        eprintln!("Erreur de validation : {}", e);
        return;
    }

    let mut state = GameState {
        current_scene_id: scenario.start_scene.clone(),
        inventory: Vec::new(),
        hp: scenario.initial_hp,
        is_running: true,
    };

    println!("Début du jeu");
    
    let _ = LookCommand.execute(&scenario, &mut state);

    while state.is_running {
        print!("\n> ");
        io::stdout().flush().unwrap(); 

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Erreur de lecture.");
            continue;
        }

        match parse_command(&input) {
            Ok(command) => {
                match command.execute(&scenario, &mut state) {
                    Ok(CommandOutcome::Moved) => {
                        let _ = LookCommand.execute(&scenario, &mut state);
                    }
                    Ok(CommandOutcome::GameOver(msg)) => {
                        println!("\n*** {} ***", msg);
                        state.is_running = false;
                    }
                    Ok(CommandOutcome::DisplayOnly) => (), 
                    Err(e) => match e {
                        GameError::InvalidChoice => println!("Choix invalide, regardez les numéros disponibles."),
                        GameError::MissingItem(item) => println!("Action impossible : il vous manque l'objet '{}'.", item),
                        GameError::SceneNotFound(id) => println!("Erreur système : la scène '{}' n'existe pas.", id),
                    },
                }
            }
            Err(e) => println!("{}", e),
        }
    }

    println!("Fin de la session.");
}
