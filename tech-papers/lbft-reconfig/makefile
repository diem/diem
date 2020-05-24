TEX = pdflatex -interaction nonstopmode

PAPER = lbft-reconfig
SUPP = mechanism.tex correctness.tex setup.tex

all: $(PAPER).pdf
	echo "all done."

clean::
	rm -fv *.aux *.log *.bbl *.blg *.toc *.out *.lot *.lof $(PAPER).pdf

$(PAPER).pdf: $(PAPER).tex $(SUPP)
	$(TEX) $(PAPER)
	$(TEX) $(PAPER)
