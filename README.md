# Connect 4 Solver 
A program that perfectly solves all possible Connect 4 positions (and my first project in rust!)

## Testing
For each improvement I made to the solver, I recorded the average time to solve (in seconds) and average number of positions searched for 6 different types of positions. 
Each of the 6 types consisted of 1000 similar positions. By tracking the improvements, we can see how effective a change is at improving the speed. 
(I also confirmed accuracy was 100%, but I won't be showing that here as it is redundant)

**Example Output**  
End-Easy: _ seconds, _ positions  
Middle-Easy: _ seconds, _ positions   
Middle_Medium: _ seconds, _ positions  
Start-Easy: _ seconds, _ positions   
Start-Medium: n/a   
Start-Hard: n/a  

If an entry is n/a, that means the solver is too slow to solve these positions in a reasonable amount of time, where "reasonable" is arbitrarily up to me.

## Version 1: Negamax
[Negamax](https://en.wikipedia.org/wiki/Negamax#:~:text=Negamax%20search%20is%20a%20variant,the%20value%20to%20player%20B.) uses depth first search to explore all branches of the game tree. It takes advantage of the fact that a good position for one player is equally bad for the opponent.
For example, if one player has a score of 5, the opponent has a score of -5. 

**Efficiency**  
End-Easy: 0.002227 seconds, 204,044 positions  
Middle-Easy: n/a   
Middle_Medium: n/a  
Start-Easy: n/a   
Start-Medium: n/a   
Start-Hard: n/a  

## Version 2: Alpha-Beta Pruning
When exploring the game tree, there are moments when we realize that our opponent would never play one branch because they have already found another that is better for them.
In these moments, we can "prune" the tree, reducing the number of positions we need to look at. This so called pruning is more formally called [alpha-beta pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning)

**Efficiency**  
End-Easy: 0.000276 seconds, 20,474 positions  
Middle-Easy: n/a   
Middle_Medium: n/a  
Start-Easy: n/a   
Start-Medium: n/a   
Start-Hard: n/a  

## Version 3: Center First Move Ordering
Alpha-beta pruning prunes the most branches when we see the best move first. Of course, we don't know the best move (that's why we are searching), but we can make some guesses.
On average, playing in a middle column is better than playing in an outside column, so we look at those moves first.

**Efficiency**  
End-Easy: 0.000136 seconds, 12,103 positions   
Middle-Easy: n/a   
Middle_Medium: n/a  
Start-Easy: n/a   
Start-Medium: n/a   
Start-Hard: n/a  

## Version 4: Optimizations
This version doesn't introduce a new feature, but rather slightly optimizes the above features. 
The two optimizations here result in an early check for a tie, and more alpha-beta pruning.
These optimizations should have been made originally, but here is their effect.

**Efficiency**  
End-Easy: 0.000038 seconds, 321 positions   
Middle-Easy: n/a   
Middle_Medium: n/a  
Start-Easy: n/a   
Start-Medium: n/a   
Start-Hard: n/a  

## Version 5: Winning Move Check
Before searching the various branches of all possible moves, we should see if any move will immediately result in a win. 
By doing this check before exploring other branches, we avoid unnecessary searches when we have a winning move elsewhere.

**Efficiency**  
End-Easy: 0.000003 seconds, 139 positions   
Middle-Easy: 0.024155 seconds, 2,081,790 positions   
Middle_Medium: 0.471179 seconds, 40,396,714 positions    
Start-Easy: n/a    
Start-Medium: n/a    
Start-Hard: n/a   

## Version 6: Early Loss and Early Move Detection
Without searching, we can determine a few things about the position (assuming we can not immediately win).    
1. If the opponent has two possible winning moves, we lost
2. If the opponent has a winning move with a threat above it, we lost. (A threat is just a place a player can win)
3. We should never play directly under an opponents threat.
4. If the opponent has a playable threat, we must play in that column.

Our solver checks for these conditions before searching, resulting in less positions searched.

**Efficiency**  
End-Easy: 0.000003 seconds, 94 positions    
Middle-Easy: 0.022833 seconds, 1,280,166 positions    
Middle_Medium: 0.456450 seconds, 24,984,148 positions    
Start-Easy: n/a     
Start-Medium: n/a     
Start-Hard: n/a    

## Version 7.1: Transposition Table
A transposition is when two branches of the game tree result in the same position. In our current solver, we search these positions everytime, but instead we can store the result
of the first search in memory, then use that recorded result if we ever find the same position. We store these results in a [transposition table](https://en.wikipedia.org/wiki/Transposition_table)

**Efficiency**  
End-Easy: 0.000009 seconds, 49 positions   
Middle-Easy: 0.001681 seconds, 40,978 positions      
Middle_Medium: 0.034100 seconds, 836,605 positions   
Start-Easy: 9.514947 seconds, 252,198,230 positions    
Start-Medium: n/a    
Start-Hard: n/a   

## Version 7.2: Larger Transposition Table
This version has a larger transposition table (1 million total entries). This means we can store more positions.

**Efficiency**  
End-Easy: 0.000427 seconds, 49 positions   
Middle-Easy: 0.001277 seconds, 16,011 positions   
Middle_Medium: 0.012167 seconds, 231,345 positions   
Start-Easy: 2.432711 seconds, 58,129,327 positions    
Start-Medium: 1.906684 seconds, 42,985,765 positions    
Start-Hard: n/a   

## Version 7.3: Faster Hash Calculation
At each node in the game tree, we call the hash function to see if the position has been searched before. By making this hash function more efficient, we make our solver faster.

**Efficiency**  
End-Easy: 0.000409 seconds, 49 positions   
Middle-Easy: 0.001208 seconds, 16,011 positions  
Middle_Medium: 0.011342 seconds, 231,345 positions   
Start-Easy: 2.308557 seconds, 58,129,327 positions   
Start-Medium: 1.799193 seconds, 42,985,765 positions    
Start-Hard: n/a

## Version 8: Iterative Deepening with Null Window Searches
When we search a position with a [null window search](https://www.chessprogramming.org/Null_Window) (beta = alpha + 1), we determine if the actual score is > alpha or <= alpha. 
These searches are very fast due to the amount of pruning that results from the small alpha-beta window.
Thus we can do many of these null window searches to hone in on the actual score until we find it.
Counterintuitively, these many null window searches are faster than one full search.

**Efficiency**  
End-Easy: 0.000402 seconds, 66 positions   
Middle-Easy: 0.000475 seconds, 1,380 positions    
Middle_Medium: 0.007325 seconds, 151,119 positions    
Start-Easy: 0.002508 seconds, 53,894 positions    
Start-Medium: 0.674147 seconds, 17,148,825 positions    
Start-Hard: n/a

## Version 9.1: Threat First Move Ordering
Previously we discussed how looking at good moves first results in a faster search. Here, in addition to prefering center moves, we also prefer moves that create threats,
because on average they will be better than moves that do not.

**Efficiency**  
End-Easy: 0.000403 seconds, 56 positions    
Middle-Easy: 0.000492 seconds, 469 positions    
Middle_Medium: 0.006484 seconds, 37,775 positions    
Start-Easy: 0.001027 seconds, 3,730 positions     
Start-Medium: 0.226199 seconds, 1,697,570 positions    
Start-Hard: 20.896086 seconds, 151,428,942 positions    

## Version 9.2: More Efficient Sorting
Looking at good moves first requires sorting. Instead of using Rust's generic sorting algorithm, we use our own insertion sort for two reasons:
1. Insertion sort is very efficient for short lists
2. With our own sorting function, we don't need to recalculate the number of threats for each comparison

**Efficiency**  
End-Easy: 0.000399 seconds, 56 positions    
Middle-Easy: 0.000453 seconds, 469 positions    
Middle_Medium: 0.003674 seconds, 37,775 positions    
Start-Easy: 0.000697 seconds, 3,730 positions    
Start-Medium: 0.119013 seconds, 1,697,570 positions    
Start-Hard: 10.664722 seconds, 151,428,942 positions    

## Version 10: Prime Transposition Table
Our transposition table is not nearly big enough to store all possible searched positions, so we use mod indexing and overwrite when needed. If our transposition table size is prime,
mod indexing will spread out the recorded values better, resulting in less overwriting and therefore more positions stored at a time. This should have been done originally.

**Efficiency**  
End-Easy: 0.000409 seconds, 56 positions   
Middle-Easy: 0.000461 seconds, 469 positions    
Middle_Medium: 0.003537 seconds, 36,089 positions   
Start-Easy: 0.000678 seconds, 3,717 positions    
Start-Medium: 0.085797 seconds, 1,265,912 positions    
Start-Hard: 6.564645 seconds, 102,314,360 positions   

## Version 11: Smaller Transposition Table Entries
Using the [Chinese Remainder Theorem](https://en.wikipedia.org/wiki/Chinese_remainder_theorem), we can still ensure no collisions despite not storing the entire position key in our transposition table.
The basic idea is that given some number n and two coprime numbers x and y, there does not exist a number m where m % x = n % x and m % y = n % y. With this optimization,
our transposition table is smaller, meaning less memory used and less overhead of managing the table.

**Efficiency**  
End-Easy: 0.000222 seconds, 56 positions
Middle-Easy: 0.000255 seconds, 469 positions
Middle_Medium: 0.003128 seconds, 36,081 positions
Start-Easy: 0.000474 seconds, 3,717 positions
Start-Medium: 0.081715 seconds, 1,265,745 positions
Start-Hard: 6.528422 seconds, 102,216,383 positions

## The Future?
Currently, I'm pretty happy with the results. I wish I could get the solve time under a seconds for the hardest set, but considering a naive approach would need to search 
trillions of positions to solve these difficult positions, 6.5 seconds doesn't seem too bad. As with any optimization problem, there's always going to be something I can do a bit
more efficiently, but for the time being I am stopping here. If I come back to this, I will probably compute multiple null window searches in parallel, allowing the solver to
hone in on the true score more efficiently. 










