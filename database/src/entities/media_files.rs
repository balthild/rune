//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use async_graphql::SimpleObject;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, SimpleObject)]
#[graphql(complex, name = "MediaFiles")]
#[sea_orm(table_name = "media_files")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub file_name: String,
    pub directory: String,
    pub extension: String,
    pub file_hash: String,
    pub last_modified: String,
    pub cover_art_id: Option<i32>,
    pub sample_rate: i32,
    #[sea_orm(column_type = "Double")]
    pub duration: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::media_analysis::Entity")]
    MediaAnalysis,
    #[sea_orm(
        belongs_to = "super::media_cover_art::Entity",
        from = "Column::CoverArtId",
        to = "super::media_cover_art::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    MediaCoverArt,
    #[sea_orm(has_one = "super::media_file_albums::Entity")]
    MediaFileAlbums,
    #[sea_orm(has_many = "super::media_file_artists::Entity")]
    MediaFileArtists,
    #[sea_orm(has_many = "super::media_file_playlists::Entity")]
    MediaFilePlaylists,
    #[sea_orm(has_many = "super::media_metadata::Entity")]
    MediaMetadata,
}

impl Related<super::media_analysis::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaAnalysis.def()
    }
}

impl Related<super::media_cover_art::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaCoverArt.def()
    }
}

impl Related<super::media_file_albums::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaFileAlbums.def()
    }
}

impl Related<super::media_file_artists::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaFileArtists.def()
    }
}

impl Related<super::media_file_playlists::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaFilePlaylists.def()
    }
}

impl Related<super::media_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MediaMetadata.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::media_analysis::Entity")]
    MediaAnalysis,
    #[sea_orm(entity = "super::media_cover_art::Entity")]
    MediaCoverArt,
    #[sea_orm(entity = "super::media_file_albums::Entity")]
    MediaFileAlbums,
    #[sea_orm(entity = "super::media_file_artists::Entity")]
    MediaFileArtists,
    #[sea_orm(entity = "super::media_file_playlists::Entity")]
    MediaFilePlaylists,
    #[sea_orm(entity = "super::media_metadata::Entity")]
    MediaMetadata,
}
