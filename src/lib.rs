#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde;
extern crate serde_json;


use hyper::Client;
use hyper::header::Connection;
use hyper::header::Headers;

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
            StockfighterErr::NoSuchVenue( _ ) => "Venue Doesn't Exist",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StockfighterVenue {
    // #[serde(default)] allows the value to be omitted from the JSON string that is returned
    // Not including this will cause an error, should the element be omitted
    #[serde(default)]
    pub venue: String,
    pub ok: bool,
    #[serde(default)]
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
    /// You can match on the return values of the heartbeat function in order
    /// to do error handling. However, a simple .unwrap() will work for most
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
    /// if test_venue.ok {
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
        let deserialized = try!(serde_json::from_str( &body ));
        mem::replace( self, deserialized );
        Ok( self.ok )
    }

    pub fn new(venue: String) -> StockfighterVenue {
        StockfighterVenue {
            venue: venue,
            ok: false,
            error: "".to_owned(),
        }
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

    pub fn stock_listing( &mut self, venue: String) -> Result<bool, StockfighterErr> {
        let url = format!("{}/venues/{}/stocks",
                          STOCKFIGHTER_API_URL.to_owned(),
                          venue);
        let mut body = String::new();
        let client = Client::new();
        let mut response = try!(client.get(&url)
                                  .header(Connection::close())
                                  .send() );
        try!( response.read_to_string( &mut body ) );
        let deserialized: StockfighterVenueStocks = try!(serde_json::from_str(&body) ); 
        mem::replace( self, deserialized );
        Ok( self.ok )
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
    pub fn heartbeat(&mut self) -> Result<bool, StockfighterErr> {
        self.ok = false;
        let url = format!("{}/heartbeat", STOCKFIGHTER_API_URL.to_owned());
        let mut body = String::new();
        let client = Client::new();
        let mut response = try!(client.get(&url)
                                 .header(Connection::close())
                                 .send() );
        try!( response.read_to_string( &mut body ) );
        let deserialized: StockfighterAPI = try!(serde_json::from_str(&body) );
        mem::replace( self, deserialized );
        Ok(self.ok)
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

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderResponse {
    pub ok: bool,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub venue: String,
    #[serde(default)]
    pub direction: String,
    #[serde(default, rename="originalQty")]
    pub original_qty: i32,
    #[serde(default)]
    pub qty: i32,
    #[serde(default)]
    pub price: i32,
    #[serde(default, rename="orderType")]
    pub order_type: String,
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub ts: String,
    #[serde(default)]
    pub fills: Vec<OrderFill>,
    #[serde(default, rename="totalFilled")]
    pub total_filled: i32,
    #[serde(default)]
    pub open: bool,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct OrderFill {
    #[serde(default)]
    pub price: i32,
    #[serde(default)]
    pub qty: i32,
    #[serde(default)]
    pub ts: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub account: String,
    pub venue: String,
    pub stock: String,
    pub price: i32,
    pub qty: i32,
    pub direction: String,
    #[serde(rename="orderType")]
    pub order_type: String,
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
            order_type: order_type,
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
        let mut body = String::new();
        try!( response.read_to_string( &mut body ));
        let deserialized = try!(serde_json::from_str( &body ));
        Ok( deserialized )

    }

}

// This would normally be an enum. However, given that we may want to try and break things later
// making it a struct will make it easier to programmatically pass something other than the four
// actual order types, but will also make it harder to accidentally make a typo.
pub struct OrderType {
    #[serde(rename="Limit")]
    limit: String,
    #[serde(rename="Market")]
    market: String,
    #[serde(rename="FillOrKill")]
    fill_or_kill: String,
    #[serde(rename="ImmediateOrCancel")]
    immediate_or_cancel: String,
}

impl OrderType {}

#[derive( Serialize, Deserialize, Debug )]
pub struct Bid {
    price: i32,
    qty: i32,
    #[serde(rename="isBuy")]
    is_buy: bool,
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
    pub fn refresh(&mut self) -> Result<bool, StockfighterErr> {
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
        mem::replace( self, deserialized );
        Ok(self.ok)
    }

    pub fn new( venue: String, stock: String ) -> OrderBook {
        OrderBook{
            ok: false,
            venue: venue,
            symbol: stock,
            bids: vec!(),
            asks: vec!(),
            ts: "".to_owned(),
        }
    }
}

#[derive( Debug, Serialize, Deserialize )]
pub struct Quote {
    pub ok: bool,
    pub symbol: String,
    pub venue: String,
    #[serde(default)]
    pub bid: i32,
    #[serde(default)]
    pub ask: i32,
    #[serde(default, rename="bidSize")]
    pub bid_size: i32,
    #[serde(default, rename="askSize")]
    pub ask_size: i32,
    #[serde(default, rename="bidDepth")]
    pub bid_depth: i32,
    #[serde(default, rename="askDepth")]
    pub ask_depth: i32,
    #[serde(default)]
    pub last: i32,
    #[serde(default, rename="lastSize")]
    pub last_size: i32,
    #[serde(default, rename="lastTrade")]
    pub last_trade: String,
    #[serde(default, rename="quoteTime")]
    pub quote_time: String,
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
            bid_size: 0,
            ask_size: 0,
            bid_depth: 0,
            ask_depth: 0,
            last: 0,
            last_size: 0,
            last_trade: "".to_owned(),
            quote_time: "".to_owned(),
        }
    }

    /// Provides information on the current valuation of a particular stock symbol at a particular
    /// venue.
    ///
    /// A quote struct needs to be created and initialized with new, and then this method is called
    /// on it.
    ///
    /// # Example
    /// ```
    /// // TESTEX is the stockfighter testing venue, and FOOBAR is the only stock that it trades
    /// let mut quote = market::Quote::new( "TESTEX".to_owned(), "FOOBAR".to_owned() );
    /// quote.get_quote().unwrap();
    /// println!("The last trade took place at {}", quote.last_trade );
    /// ```
    ///
    /// You can also match on the return value of the method to catch any errors and deal with them
    /// however you please.
    ///
    /// # Example 2
    /// ```
    /// let mut quote = market::Quote::new( "TESTEX".to_owned(), "FOOBAR".to_owned() );
    /// let response = quote.get_quote();
    /// match response {
    ///   Err( e ) => {
    ///     println!("We hit a snag: {:?}", e );
    ///   },
    ///   Ok( val ) => {
    ///      println!("Query worked. But does the venue/stock exist? This bool knows: {}", val );
    ///   }
    /// }
    /// println!("Now we can do things with the actual quote struct: {:#?}", quote );
    /// ```
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
        let deserialized: Quote = try!(serde_json::from_str(&body) );
        mem::replace( self,  deserialized );
        Ok( true )
    }

}