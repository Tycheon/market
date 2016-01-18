#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate hyper;
extern crate serde;
extern crate serde_json;


use hyper::Client;
use hyper::header::Connection;
use hyper::header::Headers;
use hyper::error::Error;

use std::io::Read;

static STOCKFIGHTER_API_URL: &'static str = "https://api.stockfighter.io/ob/api";

pub enum StockfighterErr {
  Hyper(hyper::error::Error),
  Serde(serde_json::error::Error),
}


#[derive(Serialize, Deserialize, Debug)]
pub struct StockfighterVenue {
  pub venue: String,
  pub ok: bool,
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
  /// You can catch, and match on the return values of the heartbeat function in order to do error handling.
  ///
  /// ```
  /// use market;
  /// //let mut test_venue: market::StockfighterVenue = market::StockfighterVenue{ venue: "ABCDEF".to_string(), ok: false };
  /// let mut test_venue = market::StockfighterVenue::new( "ABCDEF".to_string() );
  /// let ret = test_venue.heartbeat();
  /// match ret {
  ///   Err( e ) => {
  ///     println!("Heartbeat error: {:?}", e);
  ///     //Could be a comms error. Is likely that ABCDEF isn't a valid venue, in which case we'll get a deserialize error
  ///   },
  ///   Ok( val ) => {
  ///     println!("Hearbeat successful. Status is {:?}", val.ok);
  ///     if val.ok {
  ///       println!("We're good - Do the trading n such");
  ///     } else {
  ///       println!("We're wedged - Restart the level entirely.");
  ///     }
  ///   },
  /// }
  /// ```
  /// #Example 2
  /// Conversely, you can just ignore them and see if you get ok==true
  /// ```
  /// use market;
  /// let mut test_venue = market::StockfighterVenue::new( "ABCDEF".to_string() );
  /// test_venue.heartbeat();
  /// if test_venue.ok {
  ///   println!("Venue isn't wedged. Trade away!");
  /// }
  /// ```
  pub fn heartbeat( &mut self ) -> Result<&mut StockfighterVenue, String> {
    //DRY VIOLOATION - REFACTOR (same as our other heartbeat check)
    //Create generic and handle different errors (ie. no error field on sfvenue)
    self.ok = false;
    let url = format!("{}/venues/{}/heartbeat", STOCKFIGHTER_API_URL.to_owned(), self.venue);
    let mut body = String::new();
    let mut err: bool = false;
    let mut err_val = String::new();
    let client = Client::new();
    //Schroedinger's binding: it's both a response, and an error until you look =P
    //Only started learning rust on 13 Dec 2015, and it's currently 30 Dec 2015 - So it helps me think through the problem to 
    //Explicitly declare these
    let response: Result< hyper::client::response::Response, hyper::error::Error> = client.get( &url ).header(Connection::close()).send();
    match response {
      Err(e) => {
        self.ok = false;
        err = true;
        err_val = format!("Client.get error in StockfighterVenue::heartbeat(): {:?}\n", e);
      },
      Ok( mut the_result ) => {
        //Can't read_to_string on a Result<X,Y>, so manual error handling ftw
        the_result.read_to_string( & mut body );
      },
    }
    let deserialized: Result< StockfighterVenue, serde_json::error::Error> = serde_json::from_str( &body );
    match deserialized {
      Err(e) => { 
        self.ok = false;
        err = true;
        //Possibility for multiple error types (hyper::error::Error vs serde_json::error::Error) means try! is no good to us here either
        err_val = format!("{}Deserialize error in StockfighterVenue::heartbeat(): {:?}\n", err_val, e); 
      },
      Ok(the_result) => { 
        self.ok = the_result.ok;
      },
    }
    if err  {
      Err( err_val )
    } else {
      Ok(self)
    }
  } 

  pub fn new( venue: String ) -> StockfighterVenue {
    StockfighterVenue {
      venue: venue,
      ok: false,
    }
  }

  pub fn stock_listing( venue: String ) -> Result< Vec<Stock>, String > {
    let url = format!("{}/venues/{}/stocks", STOCKFIGHTER_API_URL.to_owned(), venue);
    let mut body = String::new();
    let mut err: bool = false;
    let mut err_val = String::new();
    let client = Client::new();
    let mut stock_list: StockfighterVenueStocks = self::StockfighterVenueStocks::new();
    let response: Result< hyper::client::response::Response, hyper::error::Error> = client.get( &url ).header(Connection::close()).send();
    match response {
      Err( e ) => {
        err = true;
        err_val = format!("client.get error in StockfighterVenueStocks::stock_listing(): {:?}\n", e);
      },
      Ok( mut the_result ) => {
        the_result.read_to_string( &mut body );
      },
    }
    let deserialized: Result< StockfighterVenueStocks, serde_json::error::Error> = serde_json::from_str( &body );
    match deserialized {
      Err(e) => {
        err = true;
        err_val = format!("{}Deserialize error in StockfighterVenueStocks::stock_listing(): {:?}\n", err_val, e);
      },
      Ok(the_result) => {
        stock_list.symbols = the_result.symbols;
      },
    }
    if err {
      Err( err_val )
    } else {
      Ok(stock_list.symbols)
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
      symbols: vec!(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StockfighterAPI {
  pub error: String,
  pub ok: bool,
}

impl StockfighterAPI {

  ///
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
  pub fn heartbeat( &mut self ) -> Result< &StockfighterAPI, String> {
    self.ok = false; 
    let url = format!("{}/heartbeat", STOCKFIGHTER_API_URL.to_owned());
    let mut body = String::new();
    let client = Client::new();
    let mut err: bool = false;
    let mut err_val = String::new();
    let response: Result< hyper::client::response::Response, hyper::error::Error> = client.get( &url ).header(Connection::close()).send();//.unwrap();
    match response {
      Err( e ) => {
        self.ok = false;
        err = true;
        err_val = format!("client.get error in StockfighterAPI::heartbeat(): {:?}\n", e);
      },
      Ok( mut the_result ) => {
        the_result.read_to_string( & mut body );
      },
    }
    let deserialized: Result< StockfighterAPI, serde_json::error::Error> = serde_json::from_str( &body );//.unwrap();
    match deserialized {
      Err( e ) => {
        self.ok = false;
        err = true;
        err_val = format!("{}Deserialize error in StockfighterAPI::heartbeat(): {:?}\n", err_val, e);
      },
      Ok( the_result ) => {
        self.ok = the_result.ok;
      },
    }
    if err {
      Err( err_val )
    } else {
      Ok( self )
    }
  } 

  pub fn new() -> StockfighterAPI {
    StockfighterAPI {
      error: "".to_string(),
      ok: false,
    }
  }  

}


/************ revise this *******************/
/**/pub struct Settings {
/**/  pub apikey: String,
/**/  pub base_url: String,
/**/  pub venue: String
/**/}
/**/
/**/impl Settings {
/**/  
/**/  pub fn new( venue: String ) -> Settings {
/**/    Settings {
/**/      apikey: self::Settings::get_apikey(),
/**/      base_url: "https://api.stockfighter.io/ob/api".to_string(),
/**/      venue: venue,
/**/    }
/**/  }
/***********************************************/
  /// Returns the API key from an environmental variable named STOCKFIGHTERAPI.
  /// Using Linux and BASH, you'd set it by adding a line like the following
  /// to the last line of your .bashrc file in your home directory (ie. cd ~/ && nano .bashrc)
  /// export STOCKFIGHTERAPI=abcdefghijklmnopqrstuvwxyzabcdefg
  /// then log out and back in.
  /// api_key_from_env() will then return that api key as a String.
  pub fn get_apikey() -> String {
    env!("STOCKFIGHTERAPI").to_string()
  }
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
  ///orderType needs to be non-snake-case, as it gets translated into a JSON field whose name is CamelCased
  orderType: String,
}

impl Order {

  pub fn new( account: String, venue: String, stock: String, price: i32, qty: i32, direction: String, order_type: String ) -> Order {
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

  pub fn encode_order( &self ) -> String {
    //let return_string = json::encode(&self).unwrap();
    let return_string = serde_json::to_string( &self ).unwrap();
    return_string.to_string() 
  }
  
  pub fn order_url( &self, the_settings: &Settings ) -> String {
    let return_string = format!("{}/venues/{}/stocks/{}/orders", the_settings.base_url, the_settings.venue, self.stock);
    return_string
  }

  /****************** needs a pub fn process_order() *************/

}

//This would normally be an enum. However, given that we may want to try and break things later
//making it a struct will make it easier to programmatically pass something other than the four
//actual order types, but will also make it harder to accidentally make a typo.
pub struct OrderType {
  Limit: String,
  Market: String,
  FillOrKill: String,
  ImmediateOrCancel: String,
}

impl OrderType {


}

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
  pub fn refresh( &mut self ) -> Result< &mut OrderBook, String > {
    self.ok = false;
    let url = format!("{}/venues/{}/stocks/{}", STOCKFIGHTER_API_URL.to_owned(), self.venue, self.symbol);
    let mut body = String::new();
    let mut err: bool = false;
    let mut err_val = String::new();
    let client = Client::new();
    let response: Result< hyper::client::response::Response, hyper::error::Error > = client.get( &url ).header(Connection::close()).send();
    match response {
      Err( e ) => {
        self.ok = false;
        err = true;
        err_val = format!("client.get error in Orderbook::refresh(): {:?}\n", e);
      },
      Ok( mut the_result ) => {
        the_result.read_to_string( &mut body );
      },
    }
    let deserialized: Result< OrderBook, serde_json::error::Error > = serde_json::from_str( &body );
    match deserialized {
      Err( e ) => {
        self.ok = false;
        err = true;
        err_val = format!("{}Deserialize error in OrderBook::refresh(): {:?}\n", err_val, e);
      },
      Ok( the_result ) => {
        self.ok = the_result.ok;
        self.bids = the_result.bids;
        self.asks = the_result.asks;
        self.ts = the_result.ts;
      },
    }
    if err {
      Err( err_val )
    } else {
      Ok( self )
    }
  }
}
