#A rust version of the command handler, using postgres

To setup the base of the database you can use the setup-db.sh script in the kafka-workshop project.

Then you need a `diesel migration run` to create the tables specific for rust, you need the diesel tool installed for this, with `cargo install diesel_cli --no-default-features --features postgres`.

To build the production build use `cargo build --release` for now some url's are hard-coded for local use.