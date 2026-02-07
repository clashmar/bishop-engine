// game/src/scripting/commands/dialogue_commands.rs
use crate::scripting::commands::lua_command::LuaCommand;
use crate::engine::Engine;
use engine_core::dialogue::SpeechBubble;
use engine_core::ecs::entity::Entity;

/// Command to show a speech bubble on an entity.
pub struct ShowSpeechCmd {
    pub entity: Entity,
    pub text: String,
    pub duration: f32,
    pub color: Option<[f32; 4]>,
    pub offset: Option<(f32, f32)>,
    pub font_size: Option<f32>,
    pub max_width: Option<f32>,
    pub show_background: Option<bool>,
    pub background_color: Option<[f32; 4]>,
}

impl LuaCommand for ShowSpeechCmd {
    fn execute(&mut self, engine: &mut Engine) {
        let mut game_state = engine.game_state.borrow_mut();
        let mut bubble = SpeechBubble::new(self.text.clone(), self.duration);

        if let Some(color) = self.color {
            bubble.color = color;
        }

        if let Some(offset) = self.offset {
            bubble.offset = offset;
        }

        if let Some(size) = self.font_size {
            bubble.font_size = Some(size);
        }

        if let Some(width) = self.max_width {
            bubble.max_width = Some(width);
        }

        if let Some(show) = self.show_background {
            bubble.show_background = show;
        }

        if let Some(bg_color) = self.background_color {
            bubble.background_color = bg_color;
        }

        game_state.game.ecs.add_component_to_entity(self.entity, bubble);
    }
}

/// Command to clear a speech bubble from an entity.
pub struct ClearSpeechCmd {
    pub entity: Entity,
}

impl LuaCommand for ClearSpeechCmd {
    fn execute(&mut self, engine: &mut Engine) {
        let mut game_state = engine.game_state.borrow_mut();
        let ecs = &mut game_state.game.ecs;
        engine_core::dialogue::clear_speech(ecs, self.entity);
    }
}

/// Command to set the current dialogue language.
pub struct SetLanguageCmd {
    pub language: String,
}

impl LuaCommand for SetLanguageCmd {
    fn execute(&mut self, engine: &mut Engine) {
        let mut game_state = engine.game_state.borrow_mut();
        game_state.game.dialogue_manager.set_language(&self.language);
    }
}
