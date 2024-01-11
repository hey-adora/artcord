use crate::database::models::auto_reaction::{
    AutoReaction, FromReactionTypeError, ToReactionTypeError,
};
use bson::oid::ObjectId;
use bson::DateTime;
use serenity::model::channel::ReactionType;
use serenity::model::id::EmojiId;

impl AutoReaction {
    pub fn to_reaction_type(self) -> Result<ReactionType, ToReactionTypeError> {
        let reaction: ReactionType = if let Some(unicode) = self.unicode {
            ReactionType::Unicode(unicode)
        } else {
            let id = self
                .id
                .ok_or(ToReactionTypeError::Id(format!("{:?}", &self._id)))?
                .parse::<u64>()?;
            let name = self
                .name
                .ok_or(ToReactionTypeError::Name(format!("{:#?}", &self._id)))?;

            ReactionType::Custom {
                animated: self.animated,
                id: EmojiId(id),
                name: Some(name),
            }
        };

        Ok(reaction)
    }

    pub fn from_reaction_type(
        guild_id: u64,
        reaction_type: ReactionType,
    ) -> Result<AutoReaction, FromReactionTypeError> {
        let auto_reaction = match reaction_type {
            serenity::model::prelude::ReactionType::Unicode(s) => {
                let auto_reaction = Self {
                    _id: ObjectId::new(),
                    guild_id: guild_id.to_string(),
                    unicode: Some(s),
                    id: None,
                    name: None,
                    animated: false,
                    modified_at: DateTime::now(),
                    created_at: DateTime::now(),
                };

                Ok(auto_reaction)
            }
            serenity::model::prelude::ReactionType::Custom { animated, id, name } => {
                let auto_reaction = Self {
                    _id: ObjectId::new(),
                    guild_id: guild_id.to_string(),
                    unicode: None,
                    id: Some(id.0.to_string()),
                    name,
                    animated,
                    modified_at: DateTime::now(),
                    created_at: DateTime::now(),
                };

                Ok(auto_reaction)
            }
            _ => Err(FromReactionTypeError::Invalid),
        }?;
        Ok(auto_reaction)
    }

    pub fn from_reaction_type_vec(
        guild_id: u64,
        reaction_types: Vec<ReactionType>,
    ) -> Result<Vec<AutoReaction>, FromReactionTypeError> {
        let mut auto_reactions: Vec<AutoReaction> = Vec::new();
        for reaction in reaction_types {
            let auto_reaction = match reaction {
                serenity::model::prelude::ReactionType::Unicode(s) => {
                    let auto_reaction = Self {
                        _id: ObjectId::new(),
                        guild_id: guild_id.to_string(),
                        unicode: Some(s),
                        id: None,
                        name: None,
                        animated: false,
                        modified_at: DateTime::now(),
                        created_at: DateTime::now(),
                    };

                    Ok(auto_reaction)
                }
                serenity::model::prelude::ReactionType::Custom { animated, id, name } => {
                    let auto_reaction = Self {
                        _id: ObjectId::new(),
                        guild_id: guild_id.to_string(),
                        unicode: None,
                        id: Some(id.0.to_string()),
                        name,
                        animated,
                        modified_at: DateTime::now(),
                        created_at: DateTime::now(),
                    };

                    Ok(auto_reaction)
                }
                _ => Err(FromReactionTypeError::Invalid),
            }?;
            auto_reactions.push(auto_reaction);
        }

        Ok(auto_reactions)
    }

    pub fn to_reaction_type_vec(
        auto_reactions: Vec<AutoReaction>,
    ) -> Result<Vec<ReactionType>, ToReactionTypeError> {
        let mut output: Vec<ReactionType> = Vec::with_capacity(auto_reactions.len());
        for reaction in auto_reactions {
            output.push(reaction.to_reaction_type()?);
        }
        Ok(output)
    }
}
