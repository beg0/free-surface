#!/usr/bin/perl
use strict;
use warnings;
# Copyright (c) 2015 - beg0

# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.


# current x and y
my ($x,$y) = (0, 0);

# number of column and rows (not used)
my ($ncols,$nrows) = (0,0);

# coordonates of top left corner
my ($xllcorner, $yllcorner) = (0, 0);

# delta in X and Y (always equal)
my ($xdelta, $ydelta) = (1, 1);

# magic value indicating no data available
my ($nodata_value) = (-9999);

# reading header
for(1..6)
{
	my $line = <>;
        $line =~ s/\r?\n$//;
	my ($key,$value) = split(/ +/,$line);

	$key = lc $key;

	if($key eq "ncols")
	{
	    $ncols = $value;
	}
	elsif($key eq "nrows")
	{
	    $nrows = $value;
	}
	elsif($key eq "xllcorner")
	{
	    $xllcorner = $value;
	}
	elsif($key eq "yllcorner")
	{
	    $yllcorner = $value;
	}
	elsif($key eq "cellsize")
	{
	    $xdelta = $ydelta = $value;
	}
	elsif($key eq "nodata_value")
	{
	    $nodata_value = $value;
	}
	else
	{
	    warn "Unknown key $key";
	}

}

# reading body
$x = $xllcorner;
$y = $yllcorner;

while( my $line = <>)
{
    $line =~ s/\r?\n$//;
    my @ys=split(/ +/,$line);
    $y = $yllcorner;
    foreach my $z (@ys)
    {
	printf("%-8d %-8d %-8d\n",$x, $y, $z) unless $z == $nodata_value;
	$y += $ydelta;
    }

    $x += $xdelta;
}
