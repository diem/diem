for file in output/*.bpl
do
  echo "verifying $file..."
  boogie /doModSetAnalysis "$file"
done
