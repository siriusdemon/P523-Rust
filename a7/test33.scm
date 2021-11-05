;;; this is the equal scheme program to the test case compile33.
;;; locals and extra parentheses are removed.
;;; start a scheme interpreter and copy-paste the code into it.
;;; I use chez scheme.
(letrec ([expt$0 (lambda (n.1 m.2)
                    (if (= m.2 1)
                        n.1
                        (* n.1 (expt$0 n.1 (- m.2 1)))))]
        [div$1 (lambda (n.1 d.2)
                    (div-helper$2 31 (- (* 2 n.1) d.2) 
                                  (* d.2 (expt$0 2 32)) 0))]
        [div-helper$2 (lambda (i.1 p.2 d.3 q.4)
                          (if (> 0 i.1)
                              q.4
                              (if (>= p.2 0)
                                  (div-helper$2 (- i.1 1)
                                                (- (* 2 p.2) d.3)
                                                d.3
                                                (logor (expt$0 2 i.1)
                                                        q.4))
                                  (div-helper$2 (- i.1 1)
                                                (- (* 2 (+ p.2 d.3)) d.3)
                                                d.3
                                                q.4))))])
  (div$1 153 17))