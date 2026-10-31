#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <inttypes.h>
typedef struct {uint64_t sequence,created_ns;} Event;
extern int32_t bwl_enqueue(uint64_t);
extern size_t bwl_drain(Event*,size_t);
extern uint64_t bwl_now_ns(void);
extern uint64_t bwl_dropped(void);
extern int32_t bwl_wait_ms(uint32_t);
static int cmp(const void *a,const void *b){uint64_t x=*(const uint64_t*)a,y=*(const uint64_t*)b;return (x>y)-(x<y);}
int main(void){
 const size_t count=200000, batch=256;
 Event events[256];uint64_t *lat=malloc(count*sizeof(uint64_t));
 if(!lat)return 3;
 size_t total=0;uint64_t prev=0;int invalid=0;
 uint64_t start=bwl_now_ns();
 for(size_t i=1;i<=count;i++){
   while(!bwl_enqueue(i)){
     size_t n=bwl_drain(events,batch);
     uint64_t now=bwl_now_ns();
     for(size_t j=0;j<n;j++){ if(events[j].sequence!=++prev)invalid++;lat[total++]=now-events[j].created_ns; }
   }
   if(i%batch==0){size_t n=bwl_drain(events,batch);uint64_t now=bwl_now_ns();
     for(size_t j=0;j<n;j++){if(events[j].sequence!=++prev)invalid++;lat[total++]=now-events[j].created_ns;}}
 }
 for(;;){size_t n=bwl_drain(events,batch);if(!n)break;uint64_t now=bwl_now_ns();
 for(size_t j=0;j<n;j++){if(events[j].sequence!=++prev)invalid++;lat[total++]=now-events[j].created_ns;}}
 uint64_t duration=bwl_now_ns()-start;
 qsort(lat,total,sizeof(uint64_t),cmp);
 printf("RESULT events=%zu invalid=%d dropped=%" PRIu64 " ns_per_event=%.1f events_per_s=%.0f delivery_ns_p50=%" PRIu64 " delivery_ns_p99=%" PRIu64 "\n",
 total,invalid,bwl_dropped(),(double)duration/total,(double)total*1e9/duration,lat[total/2],lat[(total*99)/100]);
 free(lat);
 if(invalid || total!=count || bwl_dropped())return 1;
 // Wait timeout is a functional probe, not a low-jitter benchmark.
 uint64_t t=bwl_now_ns(); if(bwl_wait_ms(2)!=0)return 2;
 printf("RESULT empty_wait_ns=%" PRIu64 "\n",bwl_now_ns()-t);
 return 0;
}
