## Mini-Redis
------
This project is simple copy of redis wich has all the data stored in the RAM and in a Json file.
The first thing the program does is defining a struct for the data of the database.
With #[derive(Debug, Clone, Serialize, Deserialize)], I allow to clone the data and transform it in or from Json with serde.
Then I define the database.
The database is defined by a string for the path of the Json file and a Rwlock<HashMap<T>> for the in memory database.
Then i define the methods of the sctruct to insert a new struct and to get a struct.
For last I define the async fuction of the microservice that allow to comunicate with an HTTP request.
In the main I initialize the db and then a Router that maps every request for my db.
the last thing to do is starting an HTTP server with axium.