Family web server that runs on a local router using Rust Actix. Monitors activity on the router and detects whether device associated with a user is streaming data. Very primitive UI to set up local users. Created to monitor my kids watching habits and provide a mechanism to allow them to watch using a point system. You can refill the points as a parent and then the points are delimited as the kids use streaming data. Very early prototype and very rough. Streaming data is not differentiated from playing games and watching videos as yet. Will be migrating to a system using eBPF for performance and better control as well as differentiating between video and games.

This project is conjuction with a custom router created with:
https://github.com/gsuyemoto/router-ubuntu-image

Which I had running on Nvidia Jetson Nano Jetson 2GB so that I could eventually run computer vision and ML on router... eventually.

Hidden file .env holds DB and server settings.

You will need Sqlite installed on the server (I used Sqlite3):<br/>
<code>sudo apt-get install sqlite3</code>

You will need Diesel CLI installed:<br/>
<code>cargo install diesel_cli --no-default-features --features sqlite</code>

Then run diesel migration in order to set up the Sqlite DB:<br/>
<code>diesel migration run</code> 
