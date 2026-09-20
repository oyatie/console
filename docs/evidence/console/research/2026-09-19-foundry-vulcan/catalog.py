import gzip,json,pathlib
from html.parser import HTMLParser
P=pathlib.Path(__file__).parent
class Tables(HTMLParser):
 def __init__(self):super().__init__();self.tables=[];self.table=None;self.row=None;self.cell=None
 def handle_starttag(self,t,a):
  if t=='table':self.table=[]
  if t=='tr' and self.table is not None:self.row=[]
  if t in ('td','th') and self.row is not None:self.cell=[]
 def handle_data(self,s):
  if self.cell is not None:self.cell.append(s)
 def handle_endtag(self,t):
  if t in ('td','th') and self.cell is not None:self.row.append(' '.join(''.join(self.cell).split()));self.cell=None
  if t=='tr' and self.row is not None:self.table.append(self.row);self.row=None
  if t=='table' and self.table is not None:self.tables.append(self.table);self.table=None
out={}
for source in ['vulcan-schema-reference','vulcan-widget']:
 t=Tables();t.feed(gzip.decompress((P/(source+'.html.gz')).read_bytes()).decode());out[source]=t.tables
(P/'schema-support-catalog.json').write_text(json.dumps({'source_tables':out,'status':'post-cutoff documentation; historical qualification and executable support not established'},indent=2)+'\n')
print({s:{'tables':len(t),'data_rows':sum(len(x)-1 for x in t)}for s,t in out.items()})
