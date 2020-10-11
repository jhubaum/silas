import sys
from .builder import SiteBuilder

site = SiteBuilder(sys.argv[1])
site.build(indir=True)
