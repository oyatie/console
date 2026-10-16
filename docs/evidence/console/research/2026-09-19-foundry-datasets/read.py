import gzip,re,sys
from html.parser import HTMLParser
from pathlib import Path
class Body(HTMLParser):
 def __init__(self):super().__init__();self.depth=0;self.parts=[]
 def handle_starttag(self,t,a):
  if ('data-pagefind-body','true') in a:self.depth=1
  elif self.depth and t=='div':self.depth+=1
  if self.depth and t in ('p','li','tr','h1','h2','h3','h4'):self.parts.append('\n')
  if self.depth and t in ('th','td'):self.parts.append(' | ')
 def handle_endtag(self,t):
  if self.depth and t=='div':self.depth-=1
 def handle_data(self,s):
  if self.depth:self.parts.append(s)
def read(path):
 p=Body();p.feed(gzip.open(path,'rt').read());return ''.join(p.parts)
if __name__=='__main__':
 for name in sys.argv[1:]:print('\nSOURCE',name,read(Path(__file__).parent/(name+'.html.gz')))
