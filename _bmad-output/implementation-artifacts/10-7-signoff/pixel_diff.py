import sys, zlib, struct


class UnsupportedPngColourType(ValueError):
    pass


def load(p):
    d=open(p,'rb').read(); i=8; w=h=None; idat=b''
    while i < len(d):
        ln=struct.unpack('>I',d[i:i+4])[0]; typ=d[i+4:i+8]; data=d[i+8:i+8+ln]
        if typ==b'IHDR': w,h,bd,ct=struct.unpack('>IIBB',data[:10])
        elif typ==b'IDAT': idat+=data
        i+=12+ln
    if ct != 2:
        raise UnsupportedPngColourType(f"{p}: unsupported PNG colour type {ct}; expected RGB (2)")
    raw=zlib.decompress(idat); bpp=3; stride=w*bpp; out=bytearray(); prev=bytearray(stride)
    pos=0
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
w,h,A=load(sys.argv[1]); _,_,B=load(sys.argv[2])
raw=d4=d16=0
for i in range(0,len(A),3):
    m=max(abs(A[i]-B[i]),abs(A[i+1]-B[i+1]),abs(A[i+2]-B[i+2]))
    if m: raw+=1
    if m>=4: d4+=1
    if m>=16: d16+=1
print(f"{sys.argv[3]:<28} raw={raw:>7,}  >=4={d4:>7,}  >=16={d16:>7,}")
