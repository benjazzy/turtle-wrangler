use sea_orm::Iterable;
use sea_orm_migration::{prelude::*, schema::*};
use turtle_types::turtle_scheme;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Turtles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Turtles::Id)
                            .primary_key()
                            .big_unsigned()
                            .not_null()
                            .to_owned(),
                    )
                    .col(string_uniq(Turtles::Name))
                    .col(enumeration(
                        Turtles::TurtleType,
                        Alias::new("type"),
                        turtle_scheme::TurtleType::iter(),
                    ))
                    .col(integer(Turtles::Fuel))
                    .col(integer(Turtles::X))
                    .col(integer(Turtles::Y))
                    .col(integer(Turtles::Z))
                    .col(enumeration(
                        Turtles::Heading,
                        Alias::new("heading"),
                        turtle_scheme::Heading::iter(),
                    ))
                    .col(timestamp(Turtles::LastSeen))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(INDEX_TURTLES_NAME)
                    .table(Turtles::Table)
                    .col(Turtles::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(INDEX_TURTLES_NAME).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Turtles::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Turtles {
    Table,
    Id,
    Name,
    TurtleType,
    Fuel,
    X,
    Y,
    Z,
    Heading,
    LastSeen,
}

const INDEX_TURTLES_NAME: &'static str = "index-turtels-name";
