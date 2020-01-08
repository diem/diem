#!/usr/local/bin/perl
use strict;

my $len;

my @foundat = ();
my $n = 0;
my $part;
my $nparts;
my $found;
my $time = 0;
while (<>) {
  if (/Looking for (\d+)-cycle/) {
    $len = $1;
    $n++;
    die unless /\/(\d+) parts/;
    $nparts = $1;
    $found = $part = 0;
    next;
  }
  if (/^(\S+)user / ) {
    $time += $1;
  } elsif (/^user\s+(\d+)m([\.\d]+)/ ) {
    $time += 60*$1 + $2;
  }
  if (!$found && /(\d+)-cycle found/) {
    my $l = $1;
    if ($l == $len) {
      $foundat[$part]++;
      $found = 1;
    }
  }
  if (/OVERLOAD/) {
    print $_;
    next;
  }
  if (/[uv]part (\d+)/) {
    die unless $1 == $part;
    $part++;
  }
}
my $quartsum = 0;
my $quartparts = 0;
my $sum = 0;
my $sumat = 0;
for my $i (0..$#foundat) {
  print "$i\t $foundat[$i]\n";
  $sum += $foundat[$i];
  if (!$quartparts && $sum >= $n/4) {
    $quartparts = $i+1;
    $quartsum = $sum;
  }
  $sumat += ($i+1) * $foundat[$i];
}
print "Total\t $sum/$n\n";
printf("Avg parts\t %.1lf/%d\n", $sumat/$sum, $nparts);
printf("Avg time\t %.1lf\n", $time/$n);
printf("Quartile parts\t (%d/%d) at %d\n", $quartsum, $n, $quartparts);
printf("Quartile time\t %.1lf\n", ($time/$n)*($quartparts/($sumat/$sum)));
