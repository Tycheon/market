#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde;
extern crate serde_json;


use hyper::Client;
use hyper::header::Connection;
use hyper::header::Headers;
//use hyper::error::Error;

use std::mem;
use std::io::Read;
use std::error::Error;
use std::fmt;

static STOCKFIGHTER_API_URL: &'static str = "https://api.stockfighter.io/ob/api";

#[derive(Debug)]
pub enum StockfighterErr {
    Hyper(hyper::error::Error),
    Serde(serde_json::error::Error),
    IO(std::io::Error),
    NoSuchVenue(String),

}

impl From<hyper::error::Error> for StockfighterErr {
    fn from( error: hyper::error::Error ) -> StockfighterErr {
        StockfighterErr::Hyper(error)
    }
}

impl From<serde_json::error::Error> for StockfighterErr {
    fn from( error: serde_json::error::Error ) -> StockfighterErr {
        StockfighterErr::Serde(error)
    }
}

impl From<std::io::Error> for StockfighterErr {
    fn from( error: std::io::Error ) -> StockfighterErr {
        StockfighterErr::IO(error)
    }
}

impl fmt::Display for StockfighterErr {
    fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
        match *self {
            StockfighterErr::Hyper( ref err ) => err.fmt(f),
            StockfighterErr::Serde( ref err ) => err.fmt(f),
            StockfighterErr::IO( ref err ) => err.fmt(f),
            StockfighterErr::NoSuchVenue( ref err ) => write!(f, "{}", err),
        }
    }
}

impl Error for StockfighterErr {
    fn description( &self ) -> &str {
        match *self {
            StockfighterErr::Hyper( ref err ) => err.description(),
            StockfighterErr::Serde( ref err ) => err.description(),
            StockfighterErr::IO( ref err ) => err.description(),
            StockfighterErr::NoSuchVenue( ref err ) => "Venue Doesn't Exist",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StockfighterVenue {
    pub venue: String,
    pub ok: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct HeartbeatErr {
    pub ok: bool,
    pub error: String,
}

impl StockfighterVenue {
    /// Venues can apparently become "wedged" (ie. deadlocked) 
    /// The heartbeat function checks the venue. If the venue responds with ok
    /// It's not deadlocked and we're okay
    ///
    /// Sets the value of the StockfighterVenue struct members based on the 
    /// response that it receives from the server.
    ///
    /// # Example
    /// You can catch, and match on the return values of the heartbeat function in order
    /// to do error handling.
    ///
    /// ```
    /// use market;
    /// let mut test_venue = market::StockfighterVenue::new( "ABCDEF".to_string() );
    /// let ret = test_venue.heartbeat();
    /// match ret {
    ///   Err( e ) => {
    ///     println!("Heartbeat error: {:?}", e);
    ///   },
    ///   Ok( val ) => {
    ///     if val {
    ///       println!("We're good - Do the trading n such");
    ///     }
    ///   },
    /// }
    /// ```
    /// #Example 2
    /// Conversely, you can just ignore them and see if you get 'true' as a return value. 
    /// ```
    /// use market;
    /// let mut test_venue = market::StockfighterVenue::new( "ABCDEF".to_string() );
    /// test_venue.heartbeat();
    /// if test_venue {
    ///   println!("Venue isn't wedged. Trade away!");
    /// }
    /// ```
    pub fn heartbeat(&mut self) -> Result<bool, StockfighterErr> {
        self.ok = false;
        let url = format!("{}/venues/{}/heartbeat",
                          STOCKFIGHTER_API_URL.to_owned(),
                          self.venue);
        let mut body = String::new();
        let client = Client::new();
        let mut response = try!(client.get(&url)
                                .header(Connection::close())
                                .send() );
        try!( response.read_to_string( &mut body ) );
        //To make things complicated, a heartbeat check returns a predictable JSON encoded struct
        //... provided that the venue exists. If it doesn't, it's a completely different JSON
        //encoded struct that gets returned: StockfighterVenue vs HeartbeatErr
        //Serde doesn't seem to have a clean method of returning one, or the other, or an Err.
        let result: Result< StockfighterVenue, serde_json::error::Error > = serde_json::from_str( &body );
        match result {
            Err( e ) => { 
                //we received a serde error ... is it because it's deserializing into a
                //HeartbeatErr struct instead of a StockfighterVenue struct?
                let val: HeartbeatErr = try!( serde_json::from_str( &body ));
                self.ok = val.ok;
                return Err( StockfighterErr::NoSuchVenue( val.error ) );
            },
            //we received a StockfighterVenue struct straight off
            Ok( val ) => { 
                self.ok = val.ok;
            },
        }
        Ok( self.ok )
    }

    pub fn new(venue: String) -> StockfighterVenue {
        StockfighterVenue {
            venue: venue,
            ok: false,
        }
    }

    pub fn stock_listing(venue: String) -> Result<Vec<Stock>, StockfighterErr> {
        let url = format!("{}/venues/{}/stocks",
                          STOCKFIGHTER_API_URL.to_owned(),
                          venue);
        let mut body = String::new();
        let client = Client::new();
        let mut stock_list: StockfighterVenueStocks = self::StockfighterVenueStocks::new();
        let mut response = try!(client.get(&url)
                                  .header(Connection::close())
                                  .send() );
        try!( response.read_to_string( &mut body ) );
        let deserialized: StockfighterVenueStocks = try!(serde_json::from_str(&body) ); 
        //populate the vector from our deserialized response
        Ok(stock_list.symbols)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StockfighterVenueStocks {
    pub ok: bool,
    pub symbols: Vec<Stock>,
}

impl StockfighterVenueStocks {
    pub fn new() -> StockfighterVenueStocks {
        StockfighterVenueStocks {
            ok: false,
            symbols: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StockfighterAPI {
    pub error: String,
    pub ok: bool,
}

impl StockfighterAPI {
    /// Checks to see if the Stockfighter API is up and running. 
    ///
    /// Sets the value of the StockfighterAPI struct members based on the 
    /// response that it receives from the server.
    ///
    /// # Example
    ///
    /// ```
    /// use market;
    /// let mut api = market::StockfighterAPI::new();
    /// api.heartbeat();
    /// if api.ok {
    ///   println!("API is UP");
    /// } else {
    ///   println!("API is DOWN\nError: {}", api.error);
    /// }
    /// ```
    pub fn heartbeat(&mut self) -> Result<&mut StockfighterAPI, StockfighterErr> {
        self.ok = false;
        let url = format!("{}/heartbeat", STOCKFIGHTER_API_URL.to_owned());
        let mut body = String::new();
        let client = Client::new();
        let mut response = try!(client.get(&url)
                                 .header(Connection::close())
                                 .send() );
        try!( response.read_to_string( &mut body ) );
        let deserialized: StockfighterAPI = try!(serde_json::from_str(&body) );
        self.ok = deserialized.ok;
        self.error = deserialized.error;
        Ok(self)
    }

    pub fn new() -> StockfighterAPI {
        StockfighterAPI {
            error: "".to_string(),
            ok: false,
        }
    }
}

pub fn get_apikey() -> String {
    env!("STOCKFIGHTERAPI").to_string()
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct OrderResponse {
    ok: bool,
    symbol: String,
    venue: String,
    direction: String,
    originalQty: i32,
    qty: i32,
    price: i32,
    orderType: String,
    id: i32,
    account: String,
    ts: String,
    fills: Vec<OrderFill>,
    totalFilled: i32,
    open: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderFill {
    price: i32,
    qty: i32,
    ts: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    account: String,
    venue: String,
    stock: String,
    price: i32,
    qty: i32,
    direction: String,
    ///orderType needs to be non-snake-case, as it gets translated into a 
    ///JSON field whose name is CamelCased
    orderType: String,
}

impl Order {
    pub fn new(account: String,
               venue: String,
               stock: String,
               price: i32,
               qty: i32,
               direction: String,
               order_type: String)
               -> Order {
        Order {
            account: account,
            venue: venue,
            stock: stock,
            price: price,
            qty: qty,
            direction: direction,
            orderType: order_type,
        }
    }

    fn encode_order(&self) -> Result< String, StockfighterErr > {
        let return_string = try!(serde_json::to_string(&self) );
        Ok( return_string.to_string() )
    }

    fn order_url(&self) -> String {
        let return_string = format!("{}/venues/{}/stocks/{}/orders",
                                    STOCKFIGHTER_API_URL.to_owned(),
                                    self.venue,
                                    self.stock);
        return_string
    }

    pub fn process_order(&self) -> Result< OrderResponse, StockfighterErr > {
        let header_vec: Vec<Vec<u8>> = vec!( get_apikey().as_bytes().to_vec() );
        let body: String = try!( self.encode_order() );
        let url = self.order_url(); 
        let mut headers = Headers::new();
        headers.set_raw("X-Starfighter-Authorization", header_vec);
        let client = Client::new();
        let mut response = try!( client.post( &url )
                                .body( &body )
                                .headers( headers )
                                .send() );
        let mut ret_string = String::new();
        try!( response.read_to_string( &mut ret_string ));
        let mut serialized_response = try!(serde_json::from_str( &ret_string ));
        Ok( serialized_response )
    }

}

// This would normally be an enum. However, given that we may want to try and break things later
// making it a struct will make it easier to programmatically pass something other than the four
// actual order types, but will also make it harder to accidentally make a typo.
#[allow(non_snake_case)]
pub struct OrderType {
    Limit: String,
    Market: String,
    FillOrKill: String,
    ImmediateOrCancel: String,
}

impl OrderType {}

#[allow(non_snake_case)]
#[derive( Serialize, Deserialize, Debug )]
pub struct Bid {
    price: i32,
    qty: i32,
    isBuy: bool,
}

#[derive( Serialize, Deserialize, Debug )]
pub struct OrderBook {
    ok: bool,
    venue: String,
    symbol: String,
    bids: Vec<Bid>,
    asks: Vec<Bid>,
    ts: String,
}

impl OrderBook {
    pub fn refresh(&mut self) -> Result<&mut OrderBook, StockfighterErr> {
        self.ok = false;
        let url = format!("{}/venues/{}/stocks/{}",
                          STOCKFIGHTER_API_URL.to_owned(),
                          self.venue,
                          self.symbol);
        let mut body = String::new();
        let client = Client::new();
        let mut response = try!(client.get(&url)
                                  .header(Connection::close())
                                  .send() );
        try!( response.read_to_string( &mut body ) );
        let deserialized: OrderBook = try!(serde_json::from_str(&body) );
        self.ok = deserialized.ok;
        self.bids = deserialized.bids;
        self.asks = deserialized.asks;
        self.ts = deserialized.ts;
        Ok(self)
    }
}

#[allow( non_snake_case )]
#[derive( Debug, Deserialize )]
pub struct Quote {
    ok: bool,
    symbol: String,
    venue: String,
    bid: i32,
    ask: i32,
    bidSize: i32,
    askSize: i32,
    bidDepth: i32,
    askDepth: i32,
    last: i32,
    lastSize: i32,
    lastTrade: String,
    quoteTime: String,
}

impl Quote {
        pub fn new( venue: String,
               symbol: String )
               -> Quote {
        Quote {
            ok: false,
            symbol: symbol.to_owned(),
            venue: venue,
            bid: 0,
            ask: 0,
            bidSize: 0,
            askSize: 0,
            bidDepth: 0,
            askDepth: 0,
            last: 0,
            lastSize: 0,
            lastTrade: "".to_owned(),
            quoteTime: "".to_owned(),
        }
    }

    pub fn get_quote( & mut self ) -> Result< bool, StockfighterErr > {
        self.ok = false;
        let url = format!("{}/venues/{}/stocks/{}/quote",
                          STOCKFIGHTER_API_URL.to_owned(),
                          self.venue,
                          self.symbol);
        let mut body = String::new();
        let client = Client::new();
        let mut response = try!(client.get(&url)
                                  .header(Connection::close())
                                  .send() );
        try!( response.read_to_string( &mut body ) );
        let mut deserialized: Quote = try!(serde_json::from_str(&body) );
        mem::replace( self,  deserialized );
        Ok( true )
    }

}