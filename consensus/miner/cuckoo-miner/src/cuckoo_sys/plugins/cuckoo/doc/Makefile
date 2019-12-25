cuckoo.pdf:	cuckoo.tex cuckoo.bbl
	pdflatex cuckoo.tex
	# The following two `pdflatex` runs are needed to resolve references of citations from bibliography. (See [Wikibooks](https://en.wikibooks.org/w/index.php?title=LaTeX/Bibliography_Management&oldid=3442012#Why_won't_LaTeX_generate_any_output?).)
	pdflatex cuckoo.tex
	pdflatex cuckoo.tex

cuckoo.bbl:	cuckoo.aux cuckoo.bib
	bibtex cuckoo

cuckoo.aux:	cuckoo.tex
	latex cuckoo.tex

