# market

market is a [rust](http://rust-lang.org/) api (currently using rust nightly) for interfacing with
[StarFighter.io](http://starfighter.io)'s [StockFighter](http://stockfighter.io) game.

# Code Example

todo

# Motivation

I've spent the last few years in IT, working everything from desk-side, to helpdesk, to server,
to networking. It's been great fun, in terms of the challenges that have been presented.
However, I often find myself yearning to get back to where I spent my university years: Programming.
I have a B.Sc. in Computer Science from the Royal Military College of Canada, 
as well as an M.Sc. in Artificial Intelligence.

[Stockfighter](http://stockfighter.io) seemed like a great way to get back into things.
Rust seemed like a perfect language to use to get back into things with (it has a lot of
what I loved in C -- previously my language of choice -- but with most of the drawbacks omitted).

I figured, why not post everything on GitHub? Maybe someone can make some use from it?

# Installation

Pull the repo to your local machine:

```
git clone https://github.com/Tycheon/market
```

Once it's been pulled down, you can `cd` into the `market` directory that was just created.

If you run `cargo build` right away, you'll get an error:

```
src/lib.rs:243:5: 243:28 error: environment variable `STOCKFIGHTERAPI` not defined
src/lib.rs:243     env!("STOCKFIGHTERAPI").to_string()
                   ^~~~~~~~~~~~~~~~~~~~~~~
error: aborting due to previous error
Could not compile `market`.
```

The market library will handle the connections to the stockfighter.io servers. However, to do so, the
library needs access to a valid Stockfighter API key. Instead of putting that API key into the code
itself, it references an environment varial named `STOCKFIGHTERAPI` which contains the api key found
at [https://www.stockfighter.io/ui/api_keys](https://www.stockfighter.io/ui/api_keys) -- we'll call
that key `<yourAPIKey>`

For *nix and Mac users, fire up the editor of your choice and insert the following line (and likely
the second one as well) in your `~/.bashrc` file:

```
export STOCKFIGHTERAPI=<yourAPIKey>
export PATH=PATH:~/.cargo/bin
```

If you're a Windows user, or don't use BASH, you'll need to figure out how to set the environment
variable in order to easily use this library.

# API Reference

todo - include rustdoc docs in repo and link to them.

# Tests

todo

# Contributors

Feel free to fork and submit pull requests. I'd also love to hear any comments regarding the code.

# License

The MIT License (MIT)

Copyright (c) 2016 @Tycheon

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

# Thanks
- jxson for his [README template](https://gist.github.com/jxson/1784669)
