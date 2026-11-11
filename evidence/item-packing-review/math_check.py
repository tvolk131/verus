"""Independent arithmetic cross-check. Exhaustive u8; boundary samples at larger widths, not a proof of those domains."""
import review as t
out={}
for w in [8,16,32,64]:
 mask=(1<<w)-1;counts=dict(zero=0,split=0,low_projection=0,field=0,three_fields=0)
 vals=lambda n:range(1<<n) if w==8 else sorted({0,1,(1<<n)//2,(1<<n)-1})
 for x in vals(w):
  t.need(((x<<0)&mask)==x and x>>0==x,'zero');counts['zero']+=1
 for k in range(1,w):
  for low in vals(k):
   for high in vals(w-k):
    p=low|((high<<k)&mask);t.need(p==low+(1<<k)*high,'sum');t.need(bool(p& (mask-((1<<k)-1)))==bool(high),'positive');counts['split']+=1
   for high in vals(w):
    p=low|((high<<k)&mask);t.need((p&((1<<k)-1))==low,'raw low');counts['low_projection']+=1
   for flag in [0,1]:t.need(bool((low|(flag<<k)) & (1<<k))==bool(flag),'boolean')
 for lo in range(w):
  for hi in range(lo+1,w+1):
   m=(1<<hi)-(1<<lo)
   for x in vals(w):t.need(((x&m)>>lo)<(1<<(hi-lo)),'bound');counts['field']+=1
   if 0<lo<hi<w:
    for low in vals(lo):
     for mid in vals(hi-lo):
      for upper in vals(w):
       p=low|((mid<<lo)&mask)|((upper<<hi)&mask)
       t.need(p&((1<<lo)-1)==low and (p&m)>>lo==mid,'three raw');counts['three_fields']+=1
 out[str(w)]=dict(coverage='exhaustive finite variables' if w==8 else 'boundary samples only',checks=counts)
t.write(t.HERE/'MATH_CHECK.json',out);print(out)
