from pathlib import Path
import re
s=Path('target/chromatic-v8-ieee754.cc').read_text()
# Diagnostic-only stand-alone build of the observed reference functions.
start=s.index('/* Get two 32 bit ints')
end=s.index('int32_t __ieee754_rem_pio2(double x, double* y)')
macros=s[start:end].replace('base::bit_cast','std::bit_cast')
functions=[]
for name in ['__ieee754_rem_pio2','__kernel_rem_pio2','__kernel_sin','__kernel_cos']:
 match=re.search(r'(?:V8_INLINE )?(?:double|int32_t|int) '+name+r'\([^;]*?\) \{',s)
 if not match:raise Exception(name)
 begin=match.start();i=match.end();depth=1
 while depth:
  depth+=int(s[i]=='{')-int(s[i]=='}');i+=1
 functions.append(s[begin:i].replace('V8_INLINE ','').replace('base::bit_cast','std::bit_cast'))
text='#include <cstdint>\n#include <cmath>\n#include <bit>\n#include <cstdio>\n#include <cstdlib>\nusing namespace std;\n'+macros+'\nint32_t __ieee754_rem_pio2(double,double*);\nint __kernel_rem_pio2(double*,double*,int,int,int,const int32_t*);\ndouble __kernel_sin(double,double,int);\ndouble __kernel_cos(double,double);\n'+'\n'.join(functions)
text+='''
double sine(double x) {double y[2];int n=__ieee754_rem_pio2(x,y);if(n==0 && y[1]==0)return __kernel_sin(y[0],0,0);switch(n&3){case 0:return __kernel_sin(y[0],y[1],1);case 1:return __kernel_cos(y[0],y[1]);case 2:return -__kernel_sin(y[0],y[1],1);default:return -__kernel_cos(y[0],y[1]);}}
int main(int argc,char** argv){for(int i=1;i<argc;i++){double x=atof(argv[i]);printf("%.17g %.17g %.17g\\n",x,sine(x),std::sin(x));}}
'''
Path('/private/tmp/chromatic-v8-probe.cc').write_text(text)
