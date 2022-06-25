Hey, this repo's a Lot!

If you wanna know how the ai works, check out misc/trainer.rs. There's basically no comments -- I wrote it really fast just to figure out the performance-intense parts of this. (It replaces a way crappier Python program that does the same.)

Ultimately, this AI just uses the non-Monte Carlo version of CFR. (See the paper [here](http://modelai.gettysburg.edu/2013/cfr/cfr.pdf).) The game tree is reduced slightly to have fewer symmetries. There's a pretty cute compression algorithm involved in reducing the size of the strategy file it ultimately generates -- you might have some fun piecing that out!

As a heads up, the strategy file for this game is about 26 MB uncompressed -- that's what the trainer script generates. So uh, God help you! You might be able to save a few bits by changing the way first moves are represented when they're written down in the infoset -- I came up with a few tricks based on that, but I'd already trained it for 200,000 iterations so I didn't want to bother changing one thing and then retraining everything.