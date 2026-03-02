mod errors;
mod models;
mod commands;

use crate::models::{Scenario, GameState, validate_scenario};
use crate::errors::GameError;
use crate::commands::{CommandOutcome, parse_command, GameCommand, LookCommand, ChooseCommand, InventoryCommand, StatusCommand, QuitCommand};

use std::io::{self, Write}; 


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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test() -> (Scenario, GameState) {
        let yaml = r#"
start_scene: start
initial_hp: 10
scenes:
  - id: start
    title: Start
    text: "Beginning"
    choices:
      - label: Go to victory
        next: win
      - label: Go to danger
        next: trap
      - label: Restricted
        next: win
        required_item: key

  - id: win
    title: Victory
    text: "You win"
    ending: Victory

  - id: trap
    title: Trap
    text: "You lose HP"
    hp_delta: -20
"#;
        let scenario: Scenario = serde_yaml::from_str(yaml).unwrap();
        let state = GameState {
            current_scene_id: scenario.start_scene.clone(),
            inventory: Vec::new(),
            hp: scenario.initial_hp,
            is_running: true,
        };
        (scenario, state)
    }

    #[test]
    fn test_victory_path() {
        let (scenario, mut state) = setup_test();
        let cmd = ChooseCommand { choice_index: 0 };
        let result = cmd.execute(&scenario, &mut state);
        
        assert!(matches!(result, Ok(CommandOutcome::GameOver(m)) if m.contains("Victory")));
        assert_eq!(state.current_scene_id, "win");
    }

    #[test]
    fn test_invalid_choice() {
        let (scenario, mut state) = setup_test();
        let cmd = ChooseCommand { choice_index: 99 };
        let result = cmd.execute(&scenario, &mut state);
        
        assert!(matches!(result, Err(GameError::InvalidChoice)));
    }

    #[test]
    fn test_missing_item() {
        let (scenario, mut state) = setup_test();
        let cmd = ChooseCommand { choice_index: 2 };
        let result = cmd.execute(&scenario, &mut state);
        
        assert!(matches!(result, Err(GameError::MissingItem(i)) if i == "key"));
    }

    #[test]
    fn test_game_over_hp() {
        let (scenario, mut state) = setup_test();
        let cmd = ChooseCommand { choice_index: 1 };
        let result = cmd.execute(&scenario, &mut state);
        
        assert!(matches!(result, Ok(CommandOutcome::GameOver(m)) if m.contains("Blessures")));
        assert!(state.hp <= 0);
    }

    #[test]
    fn test_invalid_yaml_validation() {
        let invalid_yaml = r#"
start_scene: start
initial_hp: 10
scenes:
  - id: start
    title: Start
    text: "..."
    choices:
      - label: Ghost
        next: non_existent_id
"#;
        let scenario: Scenario = serde_yaml::from_str(invalid_yaml).unwrap();
        let validation = validate_scenario(&scenario);
        
        assert!(validation.is_err());
        assert!(validation.unwrap_err().contains("non_existent_id"));
    }
}