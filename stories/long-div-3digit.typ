#import "/lib/problems/long-division.typ": long-division-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 3-digit dividend → ~6 rows of work space (2 per digit).
#long-division-problem((375, 3), opts: (answer-rows: 6))
