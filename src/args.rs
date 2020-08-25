/*
   Copyright 2020 Rupansh Sekar

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use clap::Clap;


#[derive(Clap)]
#[clap(version = "0.1.0", author = "rupansh <rupanshsekar@hotmail.com>")]
pub struct Args {
    #[clap(short, long)]
    pub hostlist: String,
    #[clap(short, long)]
    pub wordlist: String,
    #[clap(short, long)]
    pub threads: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32
}