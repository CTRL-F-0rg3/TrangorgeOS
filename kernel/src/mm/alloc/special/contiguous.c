#include "contiguous.h"
#include "../physical/pmm.h"
#include "../../arch/x86_64/memory.h"
#include "../../core/sizeutil.h"

/*
 * P1.4: `(bytes + ARCH_PAGE_SIZE - 1) / ARCH_PAGE_SIZE` byl liczony bez
 * ochrony przed przepelnieniem. Dla `bytes` bliskich SIZE_MAX dodanie
 * `ARCH_PAGE_SIZE - 1` zawijalo sie, dajac MALA wartosc `frames` —
 * wywolujacy dostawalby dalece za maly bufor fizyczny, wierzac ze ma
 * `bytes` ciaglej pamieci (blad bezpieczenstwa pamieci, nie tylko
 * kosmetyczny — pozniejszy zapis do "bufora" wychodzilby poza
 * przydzielone ramki).
 */
static bool contig_bytes_to_frames(size_t bytes, size_t *out_frames)
{
	return size_bytes_to_pages_checked(bytes, ARCH_PAGE_SIZE, out_frames);
}

bool contig_alloc(size_t bytes,
	              size_t align,
	              uint64_t *out_phys,
	              void **out_virt)
{
	if (bytes == 0) {
	    return false;
	}

	/*
	 * Kontrakt `align` spojny z kmalloc_aligned() (P1.1): 0 oznacza brak
	 * dodatkowych wymagan ponad wyrownanie do strony; niezerowa wartosc
	 * MUSI byc potega dwojki. Wczesniej dowolna wartosc byla cicho
	 * akceptowana i przepuszczana przez `arch_page_align_up()`, co dla
	 * wartosci niebedacych potega dwojki nie dawalo sensownej gwarancji
	 * wyrownania.
	 */
	if (align != 0 && !size_is_pow2(align)) {
	    return false;
	}

	size_t frames;

	if (!contig_bytes_to_frames(bytes, &frames)) {
	    return false;
	}

	size_t align_frames = 1;

	if (align > ARCH_PAGE_SIZE) {
	    /*
	     * `align` jest juz zwalidowane jako potega dwojki wieksza niz
	     * ARCH_PAGE_SIZE (tez potega dwojki), wiec dzieli sie bez
	     * reszty — nie trzeba (i nie nalezy) zaokraglac w gore przez
	     * `arch_page_align_up()`, co przy ekstremalnych `align` mogloby
	     * dawac mylace, nasycone wyniki zamiast jawnej odmowy.
	     */
	    align_frames = align / ARCH_PAGE_SIZE;
	}

	uint64_t phys = 0;

	if (!pmm_alloc_frames_aligned(frames, align_frames, &phys)) {
	    return false;
	}

	if (out_phys != NULL) {
	    *out_phys = phys;
	}

	if (out_virt != NULL) {
	    *out_virt = arch_phys_to_virt(phys);
	}

	return true;
}

void contig_free(uint64_t phys, size_t bytes)
{
	if (bytes == 0) {
	    return;
	}

	size_t frames;

	if (!contig_bytes_to_frames(bytes, &frames)) {
	    /*
	     * `bytes` nie do zwalidowania (doprowadziloby do przepelnienia)
	     * — nie ma bezpiecznego sposobu wyznaczenia liczby ramek do
	     * zwolnienia. Odmawiamy zamiast zwolnic bledna (potencjalnie
	     * zbyt mala) liczbe ramek, co odlaczyloby czesc ramek od
	     * wlasciciela bez ich faktycznego oddania do PMM (wyciek).
	     */
	    return;
	}

	pmm_free_frames(phys, frames);
}