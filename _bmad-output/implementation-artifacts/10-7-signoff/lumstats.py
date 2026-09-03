import sys, zlib, struct
def load(p):
    d=open(p,'rb').read(); i=8; w=h=None; idat=b''
    while i < len(d):
        ln=struct.unpack('>I',d[i:i+4])[0]; typ=d[i+4:i+8]; data=d[i+8:i+8+ln]
        if typ==b'IHDR': w,h,bd,ct=struct.unpack('>IIBB',data[:10])
        elif typ==b'IDAT': idat+=data
        i+=12+ln
    raw=zlib.decompress(idat); bpp=3; stride=w*bpp; out=bytearray(); prev=bytearray(stride); pos=0
    for y in range(h):
        f=raw[pos]; pos+=1; line=bytearray(raw[pos:pos+stride]); pos+=stride
        for x in range(stride):
            a=line[x-bpp] if x>=bpp else 0; b=prev[x]; c=prev[x-bpp] if x>=bpp else 0
            if f==1: line[x]=(line[x]+a)&255
            elif f==2: line[x]=(line[x]+b)&255
            elif f==3: line[x]=(line[x]+(a+b)//2)&255
            elif f==4:
                p=a+b-c; pa,pb,pc=abs(p-a),abs(p-b),abs(p-c)
                pr=a if (pa<=pb and pa<=pc) else (b if pb<=pc else c)
                line[x]=(line[x]+pr)&255
        out+=line; prev=line
    return w,h,bytes(out)
for path,label in [(a.split('=')[0], a.split('=')[1]) for a in sys.argv[1:]]:
    w,h,px = load(path)
    n = w*h; tot=0; hist=[0]*256
    for i in range(0,len(px),3):
        l = (px[i]*299 + px[i+1]*587 + px[i+2]*114)//1000
        tot += l; hist[l]+=1
    dark = sum(hist[:40]); mid = sum(hist[40:90])
    print(f"{label:<22} mean={tot/n:7.3f}  dark(<40)={dark:>7,} ({100*dark/n:5.2f}%)  shade-band(40-89)={mid:>7,} ({100*mid/n:5.2f}%)")
