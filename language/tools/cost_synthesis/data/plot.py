import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

sns.set(style="whitegrid")
graycolors = sns.mpl_palette('Set2', 2)
plt.rcParams.update({'figure.max_open_warning': 0})

df = pd.read_csv("./bytecode_instruction_costs.csv")
for col in df.columns:
    sns.distplot(df[col])

for i, col in enumerate(df.columns):
    plt.figure(i)
    df_col = df[col]
    median = df_col.median()
    mode = df_col.mode().get_values()[0]
    mean = df_col.mean()
    print 'Instruction: ', col
    print '\tMean: ', mean
    print '\tMedian: ', median
    print '\tMode: ', mode
    plt.axvline(median, color='r', linestyle='--')
    plt.axvline(mode, color='g', linestyle='-')
    plt.axvline(mean, color='b', linestyle='-')
    plt.legend({'Mean':mean, 'Mode':mode, 'Median':median})
    sns.distplot(df[col], norm_hist=True)

plt.show()
