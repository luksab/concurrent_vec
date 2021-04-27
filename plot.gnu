set terminal svg;
set terminal svg;
set output 'plot.svg';
set border 3 back;
set tics nomirror out scale 0.75
set zeroaxis lt -1;
set xtics axis;
set ytics axis;
set ylabel "inserts/s";
set xlabel "cores";
plot for [fn in system("ls plots")] "plots/".fn with lines title ''.fn
# plot 'plot' i 0 u 1:2 w lines title 'aoaVec',\
#  '' u 1:3 w lines title 'MutexVec',\
#  '' u 1:4 w lines title 'DashSet',\
#  '' u 1:5 w lines title 'AtomicInc',\
#  '' u 1:6 w lines title 'MemInc';
 #'' u 1:7 w lines title 'Increment';