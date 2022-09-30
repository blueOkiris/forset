# Forset

## Description

A Web-wielding, many-player, turn-based thinking game that runs at the bidding line.

## What's in a name?

"Forset" is not a widespread word in English, but it is __in__ English, and roughly means "strategem," a Frankish word.

Why not "strategem" though?

The newmade tongue, Anglish, which takes sight on swapping out Frankish words with those of an English root, is something that gives me mirth.

Forset is a word to wield in place of "strategem" that I quite like, so it is the name of the game.

This whole writ is done in Anglish for fun.

## How to Play

The game is played on something like an Oversoothing/Drafts board, shown at the bidding line like so:

![empty board](/screenshots/empty-board.png)

But you'll find this isn't much like Oversoothing.

First, each player will start with 5 tokens:

```
╔══════════════════════╗
║        []  <>        ║
║        ()00{}        ║
║                      ║
║                      ║
║                      ║
║                      ║
║                      ║
║                      ║
║                      ║
║        {}00()        ║
║        <>  []        ║
╚══════════════════════╝
```

Twoth, each token has errand-playing game scorings:

```
╔══════════════════════╗    player1.email@whatever.com:
║        []  <>        ║    []: H=10/11 K=SF S=10 W=10 L=0.3
║        ()00{}        ║    (): H=10/11 K=SF S=10 W=10 L=0.3
║                      ║    00: H=10/11 K=SF S=10 W=10 L=0.3
║                      ║    {}: H=10/11 K=SF S=10 W=10 L=0.3
║                      ║    <>: H=10/11 K=SF S=10 W=10 L=0.3
║                      ║
║                      ║    player2.email@whatever.com:
║                      ║    []: H=10/11 K=SF S=10 W=10 L=0.3
║                      ║    (): H=10/11 K=SF S=10 W=10 L=0.3
║        {}00()        ║    00: H=10/11 K=SF S=10 W=10 L=0.3
║        <>  []        ║    {}: H=10/11 K=SF S=10 W=10 L=0.3
╚══════════════════════╝    <>: H=10/11 K=SF S=10 W=10 L=0.3
```

Kind of scoring meanings:
- H is health which is, well, health. When it goes to naught, your token dies.
- K is weapon kind. This settles how the token plays with other tokens as well as its shifting and reach.
  1. Sword and Shield Fighters (SF) have the high ground in strength and warding over Hammerists (HM) which have it over Spear-men (SM) who have it over Sword and Shield Fighters
    - All are unsided against spell-crafting tokens
    - All are tussling short-reach tokens
  2. As for spell-crafting tokens, Fire Warlocks have high ground over Earth Witches which have it over Water Whisperers which have it over Fire Warlocks.
    - All are unsided against the tussling tokens
    - Earth Witches have the shortest reach, but the highest shifting
    - Fire Warlocks have the highest reach, but the shortest shifting
    - Water Whisperers have even reach and shifting
  3. Bowmen have bigger reach like the spell-crafting tokens, but are unsided against both tussling and spell-crafting tokens. They have reach and shifting alike that of Water Whisperers.
- S is strength which is how much root harm you do to an opponent. W is warding which is how much a token can shield against harm. When a token besets upon another, you more or less do health -= harm - defense.
  + Note: warding can be wended by kind drawbacks as spoken about before
  + Harm can be wended by the next scoring to speak about
- L is luck, and is the how likely it is to do 25% more harm

In the likeness above, I've just made a twin for each scorign, but in truly, they will be unalike.

Where do they come from then?

You make them yourself!

There's a team-building job right in the main menu where you're given a bounded set of "tines" for each token to spread out over its scorings.

You can make them even, harm-heavy, health-heavy, etc, along with setting the weapon kind to set shifting and kind drawbacks.

Once you have a team and start a game, you'll get to take turns shifting tokens and besetting.

On your turn, an errand-boy will be shown on the board which you can shift about with WASD. If you choose a token, its reach will be shown on the board and you can choose a spot from any of those markings to shift to.

Once you've shifted, if fiendish tokens are in reach, you will be able to chose among them to beset upon.

The game go on until all of your foe's tokens are dead.

## Building

Hinges on:
- \*nix kernel (Windows' bidding line doesn't work with the "termion" software-housing brooked)
- Gateway to the web
- Cargo ([the Rust build tooling](https://www.rust-lang.org/tools/install))

Run `cargo build --release; cp target/release/forset .`

