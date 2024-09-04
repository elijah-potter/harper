#! /bin/node

import fs from "fs/promises"

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

/* Randomize array in-place using Durstenfeld shuffle algorithm */
function shuffleArray(array) {
    for (var i = array.length - 1; i > 0; i--) {
        var j = Math.floor(Math.random() * (i + 1));
        var temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
}

/** @param {string} word */
async function getDefinition(word){
  while(true){
    let resp = await fetch(`https://en.wiktionary.org/api/rest_v1/page/definition/${word}`);

    if (resp.status == 429){
      sleep(60000);
      continue; 
    }

    return resp.json();
  }
}

/** @param {string} partOfSpeech  
  * @returns { string } 
  * */
function partOfSpeechToAffix(partOfSpeech){
  switch (partOfSpeech){
    case "Noun":
      return "1";
    case "Proper noun":
      return "2";
    case "Verb": 
      return "4";
    case "Adjective":
      return "5";
    case "Conjunction":
      return "7";
    case "Pronoun":
      return "8";
  }

  return ""; 
}

async function main(){
  let file = await fs.readFile("dictionary.dict", "utf8");

  let lines = file.split('\n');

  /// Remove the line count
  lines.shift();

  // This loop could probably be concurrent, but we're rate limited so it doesn't really matter.
  for (let line of lines){
    let word = line.split('\/')[0];
    let prevAffixes = line.split('\/')[1] ?? "";
    
    let def = await getDefinition(word);

    if (def.en){
      let kinds = def.en.map(v => v.partOfSpeech).map(partOfSpeechToAffix)

      // Dedup values
      kinds = [...new Set([...kinds, ...prevAffixes.split('')])];

      let affixes = kinds.reduce((prev, v) => `${prev}${v}`);

      console.log(`${word}/${affixes}`);
    }else{
      console.log(line);
    }
  }
}

main();
