# Proposal Tugas Akhir

Nov 6, 2026 · @Dzaki

## Judul Tugas Akhir

**Judul utama**

Rancang Bangun Digital Logic Forge: Simulator Rangkaian Logika Digital Visual dengan Compiled Execution untuk Komputer Buatan Pengguna

**Alternatif 1** (menonjolkan pemisahan Edit–Simulate dan proses compile)

Pengembangan Simulator Rangkaian Digital Visual Berbasis Compile-before-Run untuk Eksekusi Rangkaian Komputer Sederhana

**Alternatif 2** (menonjolkan arsitektur dan semantik yang terdokumentasi)

Implementasi Simulator Logika Digital dengan Arsitektur Compiler–Runtime Terpisah dan Semantik Simulasi Deterministik

## BAB I PENDAHULUAN

### 1.1 Latar Belakang Masalah

Setiap komputer, dari mikrokontroler sederhana hingga prosesor pada ponsel, pada dasarnya tersusun atas gerbang logika, yaitu komponen dasar yang menerima satu atau lebih sinyal bernilai 0 atau 1 lalu menghasilkan satu sinyal keluaran menurut aturan Boolean tertentu, seperti AND, OR, dan NOT. Gagasan bahwa seluruh komputer dapat dibangun dari gerbang-gerbang tersebut telah lama menjadi fondasi pendidikan arsitektur komputer. Nisan dan Schocken (2021), misalnya, menyusun pendekatan belajar yang mengajak pembelajar membangun komputer lengkap secara bertahap, dimulai dari satu jenis gerbang dasar. Untuk melakukannya tanpa perangkat keras fisik, pembelajar membutuhkan simulator rangkaian digital, yaitu perangkat lunak yang meniru perilaku rangkaian logika sehingga rancangan dapat dijalankan, diamati, dan diperbaiki sebelum dirakit secara fisik.

Praktik pengajaran memperlihatkan nilai simulator semacam itu. Ichsan dan Kurniawan (2020) merancang CPU 8-bit pada mata kuliah organisasi dan arsitektur komputer di Universitas Brawijaya dengan memakai satu simulator saja, yaitu Logisim, sebuah simulator rangkaian digital untuk pendidikan. Mereka melaporkan bahwa kelompok mahasiswa yang hanya memakai Logisim memperoleh nilai ujian lebih tinggi daripada kelompok pembanding, dan lebih dari separuhnya puas dengan proses belajar tersebut. Di Ohio Northern University, McKanna dan Hassan (2024) mengusulkan penggunaan Turing Complete, gim penyusun rangkaian, sebagai pelengkap kuliah logika digital dan arsitektur komputer. Mereka menilai perangkat lunak desain dan simulasi tingkat industri seperti Vivado dan ModelSim terlalu rumit bagi pemula, dan survei terhadap 12 mahasiswa menunjukkan tanggapan yang positif. Kedua studi berskala kecil, tetapi sejalan dalam menunjukkan bahwa simulator visual yang mudah dipakai memiliki nilai pedagogis yang nyata.

Kebutuhan akan simulator visual semacam itu kini dijawab oleh berbagai aplikasi bergaya *sandbox*, yaitu simulator yang membebaskan pengguna bereksperimen tanpa alur pembelajaran yang kaku. Digital Logic Sim karya Sebastian Lague menekankan pembangunan kompleksitas secara bertingkat melalui *chip* hierarkis, yaitu rangkaian buatan pengguna yang disimpan sebagai satu komponen lalu dipakai ulang di dalam rangkaian lain (Lague, n.d.-a). Virtual Circuit Board memisahkan mode penyusunan rangkaian dari mode simulasi dan menyediakan *assembler* serta memori virtual (*Virtual Circuit Board*, n.d.). Turing Complete mengemas perjalanan dari gerbang logika hingga bahasa *assembly* sebagai pengalaman belajar bertahap (*Turing Complete*, n.d.).

Ambisi pengguna kini melampaui rangkaian latihan seperti *counter* (rangkaian penghitung) atau unit aritmetika. Tiny Logic, produk yang dipublikasikan pada Oktober 2026, memperlihatkan arah tersebut. Pembuatnya melaporkan komputer TINY-32, prosesor 32-bit RV32IM yang disusun dari 93.338 transistor dan diklaim lulus *test suite* (kumpulan program uji) resmi RISC-V (PlangoDev, n.d.; whoismuzzy, 2026). RISC-V adalah arsitektur set instruksi terbuka, yakni spesifikasi instruksi yang dipahami sebuah CPU (central processing unit, yaitu pemroses utama komputer), dan RV32IM adalah varian 32-bit yang dilengkapi instruksi perkalian dan pembagian bilangan bulat. Repositori demonstrasinya juga memuat program DOOM yang dijalankan di atas komputer tersebut (PlangoDev, n.d.). Angka-angka itu merupakan klaim pembuatnya dan belum diverifikasi secara independen, tetapi pesannya jelas, yaitu pengguna ingin menjalankan perangkat lunak sungguhan di atas komputer yang mereka rakit sendiri.

Ambisi tersebut mengubah kecepatan simulator dari sekadar kenyamanan menjadi syarat kelayakan. Satu siklus (*cycle*) adalah satu langkah sinkron seluruh rangkaian mengikuti *clock*, yaitu sinyal berulang yang menjadi penanda waktu bagi komponen penyimpan seperti register. Sebuah CPU buatan pengguna harus melalui siklus dalam jumlah besar agar program sederhana pun terasa interaktif. Kecepatan simulator diukur dengan *cycles per second* (CPS), sedangkan kelancaran aplikasi seperti permainan diukur dengan *frames per second* (FPS). Keduanya tidak setara karena satu *frame* menuntut banyak siklus CPU untuk menjalankan instruksi program, memperbarui memori tampilan, dan baru kemudian digambar. Simulator yang cepat pada rangkaian kecil karenanya belum tentu layak untuk komputer buatan pengguna.

Literatur simulasi logika digital telah lama membahas cara mencapai kecepatan tersebut. Gunes et al. (2005) membedakan dua mekanisme dasar. Pada *compiled simulation*, rangkaian lebih dahulu diubah menjadi urutan instruksi tetap yang tinggal dieksekusi saat simulasi berjalan. Pada *event-driven simulation*, simulator memelihara *event queue* (antrean perubahan sinyal yang menunggu diproses) dan hanya menghitung ulang komponen yang inputnya berubah. Pendekatan kedua tampak lebih hemat karena sebagian besar rangkaian tidak berubah pada setiap saat, tetapi mencatat dan menjadwalkan perubahan juga memerlukan biaya. Wang dan Maurer (1990) melaporkan bahwa LECSIM, simulator yang memadukan *compiled execution* dengan *levelization* (pengurutan gerbang menurut ketergantungannya sehingga setiap gerbang dihitung setelah semua inputnya siap), berjalan sekitar 8 hingga 77 kali lebih cepat daripada simulator *event-driven* konvensional yang bersifat *interpreted* pada *benchmark* mereka. Maurer (2003) menunjukkan bahwa keunggulan salah satu pendekatan sangat bergantung pada tingkat aktivitas rangkaian, yakni seberapa banyak sinyal berubah per satuan waktu. Beamer dan Donofrio (2020) menambahkan bahwa simulator rangkaian pada level *register-transfer* (RTL) modern sering tetap menghitung seluruh rangkaian di setiap siklus, karena biaya mendeteksi perubahan dapat melampaui biaya menghitung ulang. Angka-angka tersebut spesifik pada *benchmark* dan perangkat keras zamannya, sehingga bukan prediksi kinerja untuk simulator baru. Pelajarannya adalah tidak ada pendekatan yang otomatis unggul, sehingga pilihan harus ditentukan oleh *workload*, yaitu jenis beban yang dijalankan simulator, dan diukur.

Kajian awal terhadap lima simulator pembanding, melalui dokumentasi publik dan inspeksi *source code* bila tersedia, memperlihatkan bahwa masing-masing menyelesaikan sebagian masalah ini. Digital Logic Sim, yang *source code*-nya diinspeksi pada 5 November 2026, memisahkan deskripsi *chip* dari representasi *runtime*, yaitu bentuk data yang dipakai saat simulasi berjalan. Namun simulator ini memproses hierarki *chip* secara rekursif, dan ketika tidak ada *chip* yang inputnya siap akibat *feedback* (keluaran yang disalurkan kembali menjadi masukan rangkaian yang sama), ia memilih salah satu *chip* secara acak (Lague, n.d.-b). Logisim Evolution memiliki *event queue* dan semantik *propagation* (perambatan sinyal) yang eksplisit, tetapi dirancang untuk simulasi interaktif dan pendidikan, bukan untuk memaksimalkan *throughput* (jumlah pekerjaan yang diselesaikan per satuan waktu) pada komputer buatan pengguna (Logisim-evolution, n.d.). Virtual Circuit Board menyajikan alur Edit dan Simulate yang jelas serta mengklaim kecepatan hingga jutaan *update* per detik, tetapi sebagai perangkat lunak tertutup, mekanisme internal dan angka kinerjanya tidak dapat diverifikasi (*Virtual Circuit Board*, n.d.). Turing Complete berorientasi pada *puzzle* dan pembelajaran bertahap, bukan pada simulasi berkinerja tinggi (*Turing Complete*, n.d.). Tiny Logic menargetkan simulasi hingga level transistor dan perilaku elektris dengan *engine* simulasi yang tidak terbuka (whoismuzzy, 2026), sehingga cakupannya jauh lebih luas daripada simulasi CPU digital.

Perilaku acak pada Digital Logic Sim menunjuk pada masalah yang lebih mendasar daripada kecepatan, yaitu semantik simulasi, berupa aturan yang menentukan secara pasti apa yang terjadi pada rangkaian. Simulator yang hasilnya bergantung pada urutan komponen di *file* atau pada pilihan acak sulit dipercaya untuk membangun komputer, sebab pengguna tidak dapat membedakan apakah sebuah kesalahan berasal dari rancangannya atau dari simulator. Aturan itu mencakup, antara lain, kapan register menyimpan nilai baru pada *clock edge* (saat *clock* beralih dari 0 ke 1 atau sebaliknya), apa yang terjadi bila dua keluaran mengemudikan satu *wire* (kawat penghubung) yang sama, dan bagaimana nilai yang belum diketahui direpresentasikan. Untuk yang terakhir, simulator digital lazim memakai nilai X (tidak diketahui atau terjadi konflik) dan Z (*high impedance*, yaitu keluaran yang sedang tidak memaksakan nilai pada *wire*), di samping 0 dan 1.

Berdasarkan gambaran tersebut, diusulkan pengembangan Digital Logic Forge (DLF), simulator rangkaian logika digital visual yang dirancang untuk komputer buatan pengguna. DLF mengambil pengalaman menyusun rangkaian secara visual seperti pada Digital Logic Sim, dengan hierarki komponen buatan pengguna sebagai target penguatan, serta memisahkan mode kerja seperti pada Virtual Circuit Board. Pada mode Edit, pengguna menyusun dan mengubah rangkaian. Ketika menekan Simulate, DLF memvalidasi rangkaian, menyelesaikan koneksinya, menyusun urutan evaluasi, dan membangun representasi *runtime* yang padat dalam satu tahap *compile*, yaitu proses menerjemahkan rangkaian menjadi bentuk yang siap dijalankan, sehingga pekerjaan yang mahal dibayar sekali, bukan pada setiap siklus. Rangkaian kemudian dijalankan oleh *simulation worker*, yaitu alur kerja latar belakang (*background thread*) yang terpisah dari antarmuka pengguna dan menjadi satu-satunya pemilik data *state* simulasi (keadaan nilai sinyal dan komponen penyimpan). UI hanya menerima salinan hasil (*snapshot*), sehingga tetap responsif tanpa memengaruhi hasil simulasi. DLF juga menetapkan semantik yang eksplisit dan deterministik, dan menolak rangkaian yang belum didukung dengan diagnostik alih-alih menghasilkan hasil yang diam-diam salah.

Posisi DLF dipilih secara sadar. DLF tidak berusaha menyaingi Tiny Logic dalam kedalaman fisik karena simulasi transistor dan listrik analog memperluas cakupan secara drastis dan bukan syarat untuk menjalankan CPU digital. DLF berfokus pada tingkat gerbang dan komponen digital, dengan keunggulan yang dapat dipertanggungjawabkan pada arsitektur *compile-before-run*, semantik yang terdokumentasi, serta *benchmark* yang transparan dan dapat diulang.

Nilai proyek ini tidak berhenti pada terwujudnya aplikasi, tetapi pada cara klaim-klaimnya dipertanggungjawabkan. Kebenaran hasil ditempatkan sebagai gerbang sebelum optimasi, melalui *reference evaluator*, yaitu evaluator sederhana yang dipakai sebagai pembanding hasil, dan *test suite* semantik. Kinerja dilaporkan terpisah per aspek, yaitu waktu *compile*, kecepatan inti simulasi, penggunaan memori, daya tanggap UI (*responsiveness*), dan FPS aplikasi demonstrasi, dengan protokol pengukuran yang dapat diulang (Beyer et al., 2019). Hasilnya adalah produk perangkat lunak yang dapat dipakai sekaligus bukti rekayasa yang memungkinkan keputusan arsitekturnya ditinjau ulang. Pada saat proposal ini disusun, kajian arsitektur telah selesai tetapi implementasi produk belum dimulai, sehingga seluruh target kinerja diperlakukan sebagai asumsi rancangan yang akan diuji, bukan sebagai klaim.

### 1.2 Identifikasi Masalah

Dari uraian latar belakang, ditemukan tujuh permasalahan yang mendasari proyek ini.

1. Simulator visual yang ada belum memadukan pengalaman menyusun rangkaian yang hierarkis, tahap *compile* yang eksplisit, semantik yang terdokumentasi, dan *benchmark* yang transparan dalam satu perangkat lunak. Dua produk yang paling dekat dengan kebutuhan komputer buatan pengguna bersifat tertutup, sehingga mekanisme internal dan klaim kinerjanya tidak dapat diperiksa.
2. Menjalankan rangkaian langsung dari struktur data *editor*, termasuk menelusuri hierarki *chip* secara rekursif pada setiap siklus, menambah beban pada jalur eksekusi yang paling sering dipanggil (*hot loop*). Belum ada pemisahan yang tegas antara model penyuntingan, hasil *compile*, dan *runtime state*.
3. Semantik simulasi pada sebagian simulator tidak sepenuhnya deterministik. Penyelesaian *feedback* dengan pilihan acak, konflik beberapa keluaran pada satu *wire*, nilai X dan Z, saat register menyimpan nilai baru, serta perilaku memori saat dibaca dan ditulis bersamaan belum dirumuskan sebagai aturan yang konsisten dan dapat diuji.
4. Teknik *scheduling* (pengaturan urutan evaluasi komponen) yang paling sesuai untuk rangkaian sinkron seperti CPU belum jelas. Literatur menunjukkan bahwa keunggulan pendekatan *event-driven* atau *levelized compiled* bergantung pada aktivitas rangkaian (Maurer, 2003; Wang & Maurer, 1990), sehingga keputusan tidak dapat diambil berdasarkan intuisi.
5. UI dan simulasi yang berbagi data *state* tanpa batas komunikasi yang jelas berisiko menimbulkan hasil yang tidak konsisten dan UI yang tidak responsif, terutama saat simulasi berjalan dalam jumlah siklus besar.
6. Klaim kecepatan sulit dibandingkan karena definisi "*update*" atau siklus, ukuran rangkaian, dan perangkat uji tidak seragam. Kecepatan inti simulasi (CPS) juga sering disamakan dengan kelancaran aplikasi (FPS), padahal keduanya berbeda dan perlu diukur dengan protokol yang dapat diulang (Beyer et al., 2019).
7. Cakupan simulator mudah membesar, misalnya hingga simulasi transistor atau perilaku listrik analog. Dengan sumber daya pengembang yang terbatas, proyek memerlukan batas ruang lingkup yang tegas agar inti simulator tetap selesai dan teruji.

### 1.3 Batasan Masalah

Agar proyek dapat diselesaikan dengan sumber daya satu pengembang dan hasilnya dapat diuji, ruang lingkup dibatasi sebagai berikut.

1. **Tingkat simulasi.** DLF mensimulasikan gerbang dan komponen digital. DLF tidak mensimulasikan transistor atau perilaku listrik analog (seperti pada simulator SPICE), dan tidak menjanjikan *propagation delay* fisik, *setup/hold time* (syarat kestabilan data di sekitar *clock edge*), *metastability* (keadaan register yang tidak menentu akibat pelanggaran syarat tersebut), maupun konflik listrik yang realistis.
2. **Model waktu.** Simulasi bersifat fungsional berbasis siklus (*cycle-based*), yaitu logika kombinasional diselesaikan dalam satu siklus, lalu elemen penyimpan memperbarui nilainya pada batas *clock*. Inti proyek mencakup satu domain *clock*. Beberapa domain *clock* dan kendali asinkron yang kompleks tidak termasuk.
3. **Bukan pengganti simulator HDL.** DLF bukan pengganti simulator bahasa deskripsi perangkat keras (*hardware description language*, HDL) seperti Verilog atau SystemVerilog, dan bukan pengganti perangkat bantu pengembangan FPGA (field-programmable gate array, yaitu chip yang rangkaian logikanya dapat diprogram ulang).
4. **Cakupan fungsional inti.** Produk minimum mencakup dokumen rangkaian yang dapat disimpan dan dimuat, *editor* dasar (*canvas*, pindah, hubungkan, dan pengaturan properti), *library* komponen primitif, resolusi konektivitas, tahap *compile*, *runtime* dengan *static evaluation schedule*, komponen sekuensial (register), memori RAM dan ROM, *simulation worker* terpisah, kendali simulasi (*run*, *pause*, *step*, *reset*, *stop*), *observability* sinyal, validasi kebenaran, dan *benchmark*. Target penguatan berikutnya meliputi *chip* hierarkis yang dapat dipakai ulang, logika empat nilai penuh (0, 1, X, Z), dukungan *feedback*, bus multi-bit hingga 64 bit, serta penguatan *simulation worker* melalui *batching* yang dapat disetel dan penggabungan *snapshot*.
5. **Perubahan struktur saat simulasi.** Struktur rangkaian tidak dapat diubah selama simulasi berjalan. Pengguna kembali ke mode Edit lalu melakukan *compile* ulang. Masukan *runtime* seperti *switch*, tombol, dan *keyboard* tetap dapat diubah tanpa *compile* ulang.
6. **Teknik yang ditunda.** *Compile* penuh dari *snapshot* menjadi dasar awal. *Just-in-time compilation* (JIT, pembuatan *machine code* saat program berjalan) atau *native code generation*, *incremental compilation*, API *plugin* publik, serta eksekusi paralel atau GPU berada di luar cakupan dan hanya dipertimbangkan bila hasil pengukuran membuktikan kebutuhannya.
7. **Komputer demonstrasi.** Subset RISC-V menjadi target demonstrasi CPU buatan pengguna. DLF tidak menyediakan *assembler* atau *compiler* sebagai fitur produk. Program *Snake* atau *Pong* menjadi target demonstrasi, dan bila tidak tercapai, program deterministik yang lebih kecil digunakan sebagai cadangan yang dinyatakan secara eksplisit, bukan disamarkan sebagai keberhasilan.
8. **Platform.** Aplikasi ditargetkan untuk desktop Windows 64-bit. UI dibangun dengan Godot, yaitu *engine* pembuat aplikasi dan permainan sumber terbuka, sedangkan inti simulasi ditulis dalam Rust, yaitu bahasa pemrograman sistem yang menekankan keamanan memori. Pilihan ini merupakan landasan implementasi sementara yang masih harus divalidasi melalui *prototype*. Linux dan macOS tidak menjadi target.
9. **Evaluasi kinerja.** Pengukuran dilakukan pada perangkat uji yang dicatat spesifikasinya dan tidak diklaim sebagai peringkat universal. Simulator lain dijadikan pembanding numerik hanya bila *workload*, semantik, dan definisi siklusnya dapat disejajarkan. Klaim kinerja dari pengembang produk lain tidak dipakai sebagai angka pembanding.
10. **Fitur di luar cakupan.** Rekaman *waveform* penuh untuk semua sinyal pada setiap siklus, kolaborasi dan sinkronisasi *cloud*, serta kompatibilitas migrasi *file* lintas banyak versi tidak termasuk.

### 1.4 Rumusan Masalah

Berdasarkan identifikasi masalah dan batasan di atas, rumusan masalah proyek ini adalah sebagai berikut.

1. Bagaimana merancang arsitektur DLF yang memisahkan dokumen penyuntingan, hasil *compile*, dan *runtime state*, sehingga rangkaian dapat diubah menjadi *execution plan* yang ringkas melalui tahap *compile*?
2. Bagaimana merumuskan dan mengimplementasikan semantik simulasi yang deterministik, mencakup nilai sinyal 0, 1, X, dan Z, penyelesaian beberapa keluaran pada satu *wire*, *feedback*, pembaruan register pada *clock edge*, serta perilaku memori?
3. Bagaimana mengimplementasikan *runtime* dengan *evaluation schedule* berurutan menurut level ketergantungan, serta *simulation worker* yang menjaga UI tetap responsif tanpa memengaruhi hasil simulasi?
4. Bagaimana membangun *editor* visual dasar yang memungkinkan pengguna menyusun, menyimpan, dan menjalankan rangkaian tanpa menyunting *file* secara manual, dengan tetap memisahkan tampilan gambar rangkaian dari data hubungan antarkomponen yang dipakai simulator?
5. Bagaimana memverifikasi kebenaran hasil simulasi dan mengukur kinerja DLF, yaitu waktu *compile*, kecepatan inti simulasi, penggunaan memori, *responsiveness* UI, dan FPS aplikasi demonstrasi, secara terpisah dan dapat diulang?
6. Bagaimana mendemonstrasikan bahwa DLF mampu menjalankan CPU sederhana buatan pengguna yang menjalankan program deterministik, termasuk jalur masukan dan tampilan?

### 1.5 Tujuan Tugas Akhir

Tugas Akhir ini berbasis proyek, sehingga tujuannya dirumuskan sebagai luaran rancangan dan implementasi yang dapat diperiksa, bukan sebagai pengujian hipotesis. Tujuan 1 sampai 5 merupakan komitmen inti pada prioritas P0 (kebutuhan minimum yang harus selesai, dirinci pada subbab 3.2), sedangkan Tujuan 6 beserta perluasannya, yaitu hierarki komponen kustom, logika empat nilai penuh, dan penyelesaian feedback, merupakan target prioritas P1 (penguatan yang dikerjakan setelah inti stabil) yang hasilnya dilaporkan apa adanya. Tujuan yang ingin dicapai adalah sebagai berikut.

1. Merancang arsitektur DLF yang memisahkan dokumen penyuntingan, hasil *compile* yang tidak dapat diubah (*immutable*), dan *runtime state*, lengkap dengan alur *compile* dari rangkaian visual menjadi *execution plan* yang ringkas.
2. Menetapkan semantik simulasi yang deterministik dan terdokumentasi, termasuk nilai 0, 1, X, dan Z, mengimplementasikannya secara bertahap mulai dari cakupan inti, serta menolak rangkaian yang belum didukung dengan diagnostik yang dapat ditelusuri ke objek visualnya.
3. Mengimplementasikan *runtime* dengan *evaluation schedule* berbasis level ketergantungan, pembaruan *state* serentak pada batas *clock*, penyimpanan memori, dan *simulation worker* terpisah yang berkomunikasi dengan UI melalui perintah dan *snapshot*.
4. Membangun *editor* visual dasar yang memungkinkan pengguna menyusun, menyimpan, dan menjalankan rangkaian tanpa menyunting *file* secara manual.
5. Menyusun dan menjalankan *test suite* kebenaran serta protokol *benchmark* yang memisahkan pengukuran waktu *compile*, kecepatan inti simulasi, memori, *responsiveness* UI, dan FPS aplikasi demonstrasi.
6. Mendemonstrasikan CPU sederhana buatan pengguna yang menjalankan program deterministik, dengan hasil dan keterbatasannya dilaporkan apa adanya.

### 1.6 Manfaat Tugas Akhir

**Bagi pengguna dan pendidik.** Pembelajar arsitektur komputer memperoleh simulator visual yang perilakunya konsisten dan terdokumentasi, sehingga kesalahan pada rangkaian dapat dibedakan dari kesalahan simulator. Pendidik dapat memakainya untuk menunjukkan jalur dari gerbang logika hingga CPU yang menjalankan program.

**Bagi pengembang dan komunitas simulator.** Keputusan arsitektur, semantik, dan metode pengukuran dicatat beserta alasan dan batas buktinya. Dokumentasi ini dapat menjadi acuan bagi pengembang simulator sejenis yang menghadapi pilihan *scheduling*, penyimpanan sinyal, dan pemisahan UI dari *runtime*.

**Bagi keilmuan rekayasa perangkat lunak.** Proyek ini menyediakan studi kasus penerapan teknik simulasi logika yang dikenal dalam literatur pada aplikasi visual interaktif modern, lengkap dengan protokol evaluasi yang dapat diulang serta batas bukti yang dinyatakan secara eksplisit.

**Bagi penulis.** Proyek ini melatih perancangan sistem perangkat lunak secara utuh, mulai dari analisis kebutuhan, perancangan arsitektur, implementasi, pengujian, hingga evaluasi kinerja, serta pengelolaan risiko dan ruang lingkup pada proyek berskala besar.

## Referensi Sementara Bab I

Beamer, S., & Donofrio, D. (2020). Efficiently exploiting low activity factors to accelerate RTL simulation. In *2020 57th ACM/IEEE Design Automation Conference (DAC)* (pp. 1–6). IEEE. https://doi.org/10.1109/DAC18072.2020.9218632

Beyer, D., Löwe, S., & Wendler, P. (2019). Reliable benchmarking: Requirements and solutions. *International Journal on Software Tools for Technology Transfer, 21*(1), 1–29. https://doi.org/10.1007/s10009-017-0469-y

Gunes, M., Thornton, M. A., Kocan, F., & Szygenda, S. A. (2005). A survey and comparison of digital logic simulators. In *2005 48th Midwest Symposium on Circuits and Systems* (pp. 744–749). IEEE. https://doi.org/10.1109/MWSCAS.2005.1594208

Ichsan, M. H. H., & Kurniawan, W. (2020). CPU implementation using only Logisim simulator to achieve computer architecture learning outcome. *Bulletin of Electrical Engineering and Informatics, 9*(2), 747–754. https://doi.org/10.11591/eei.v9i2.1972

Lague, S. (n.d.-a). *Digital Logic Sim* \[Computer software\]. itch.io. https://sebastian.itch.io/digital-logic-sim

Lague, S. (n.d.-b). *Digital-Logic-Sim* \[Source code\]. GitHub. Retrieved November 5, 2026, from https://github.com/SebLague/Digital-Logic-Sim

Logisim-evolution. (n.d.). *Logisim-evolution* \[Source code\]. GitHub. https://github.com/logisim-evolution/logisim-evolution

Maurer, P. M. (2003). Efficient event-driven simulation by exploiting the output observability of gate clusters. *IEEE Transactions on Computer-Aided Design of Integrated Circuits and Systems, 22*(11), 1471–1486. https://doi.org/10.1109/TCAD.2003.818305

McKanna, E., & Hassan, F. (2024). Enhancing student understanding of digital logic and computer architecture through Turing Complete game challenges. In *Proceedings of the 2024 ASEE North Central Section Conference*. American Society for Engineering Education. https://peer.asee.org/45615

Nisan, N., & Schocken, S. (2021). *The elements of computing systems: Building a modern computer from first principles* (2nd ed.). The MIT Press.

PlangoDev. (n.d.). *tiny-logic-doom* \[Source code\]. GitHub. https://github.com/PlangoDev/tiny-logic-doom

*Turing Complete*. (n.d.). https://turingcomplete.game/

*Virtual Circuit Board*. (n.d.). Steam. https://store.steampowered.com/app/1885690/Virtual\_Circuit\_Board/

Wang, Z., & Maurer, P. M. (1990). LECSIM: A levelized event driven compiled logic simulation. In *Proceedings of the 27th ACM/IEEE Design Automation Conference* (pp. 491–496). IEEE Computer Society Press. https://doi.org/10.1145/123186.123349

whoismuzzy. (2026). *Tiny Logic* \[Early access video game\]. itch.io. https://whoismuzzy.itch.io/tinylogic

## BAB II TINJAUAN PUSTAKA

### 2.1 Landasan Teori Rangkaian Digital dan Semantik Simulasi

Sebuah simulator rangkaian digital pada dasarnya adalah penerjemah aturan. Rangkaian yang digambar pengguna hanya berarti sesuatu bila ada aturan yang pasti tentang nilai apa yang dibawa sinyal, bagaimana nilai itu bergabung, dan kapan komponen penyimpan berubah. Kumpulan aturan inilah yang disebut semantik simulasi. Subbab ini menguraikan konsep dasar yang menjadi bahan penyusunan semantik tersebut.

#### 2.1.1 Rangkaian Kombinasional dan Sekuensial

Rangkaian digital dibedakan menjadi dua jenis. Pada rangkaian kombinasional, keluaran hanya bergantung pada masukan saat ini. Contohnya *adder* (penjumlah) dan ALU (*arithmetic logic unit*), yaitu bagian CPU yang melakukan operasi aritmetika dan logika. Pada rangkaian sekuensial, keluaran juga bergantung pada *state* (keadaan) yang tersimpan, misalnya pada register, *counter*, dan memori. Perubahan *state* diatur oleh *clock*, yaitu sinyal berulang yang naik dan turun secara teratur. Peralihan dari 0 ke 1 disebut *rising edge*, sedangkan peralihan dari 1 ke 0 disebut *falling edge*.

Komponen sekuensial yang dipicu oleh *clock edge* disebut *flip-flop*, sedangkan *latch* bersifat peka level. Saat sinyal pengaktifnya aktif, *latch* bersifat transparan sehingga keluarannya mengikuti masukan, dan saat tidak aktif ia mempertahankan nilai sebelumnya. Dalam CPU, banyak register berubah pada *clock edge* yang sama. Agar hasilnya tidak bergantung pada urutan pemrosesan register di dalam simulator, semua register harus membaca *state* lama yang sama sebelum seluruh *state* baru disimpan secara serentak (*simultaneous commit*). Tanpa aturan ini, register yang diproses lebih dahulu dapat memengaruhi register lain pada *edge* yang sama.

#### 2.1.2 Nilai Sinyal, Net, dan Resolusi Beberapa Keluaran

Simulator paling sederhana hanya mengenal 0 dan 1. Standar IEEE 1164 untuk bahasa deskripsi perangkat keras VHDL memakai sembilan nilai, termasuk nilai tidak diketahui, tidak diinisialisasi, dan *high impedance* (IEEE, 1993). Untuk simulator visual, empat nilai per bit sudah mencukupi, yaitu 0, 1, X, dan Z. Nilai X berarti tidak diketahui atau terjadi konflik, misalnya pada register yang belum diisi. Nilai Z berarti *high impedance*, yaitu sebuah keluaran sedang tidak memaksakan nilai apa pun pada wire. Keduanya tidak boleh digabung karena X menyatakan ketidakpastian nilai, sedangkan Z menyatakan keadaan penggerak.

Gerbang logika empat nilai harus menjalankan aturan yang konservatif. Hasil 0 AND X tetap 0 karena hasilnya sudah pasti, sedangkan 1 AND X menghasilkan X. Nilai Z yang masuk ke gerbang biasa diperlakukan sebagai X. Dengan aturan ini, ketidaktahuan tidak diam-diam berubah menjadi 0 atau 1. Sebaliknya, Verilator, simulator HDL berkinerja tinggi, sengaja memakai model yang sebagian besar dua nilai dan mendukung X serta Z hanya pada konteks terpilih (Verilator, n.d.-a). Perbandingan ini menunjukkan bahwa model nilai merupakan pertukaran antara ketelitian dan kecepatan.

Sebuah net, yaitu himpunan titik yang terhubung secara listrik, dapat digerakkan oleh nol, satu, atau beberapa keluaran (*driver*). Keluaran yang membaca nilai net disebut *sink*. Untuk beberapa *driver*, nilai net ditentukan oleh fungsi resolusi seperti yang dikenal pada IEEE 1164 (IEEE, 1993), dengan aturan sederhana pada Tabel 2.1.

**Tabel 2.1** Resolusi beberapa *driver* pada satu bit net

| Kondisi *driver* aktif | Nilai net |
| --- | --- |
| Tidak ada *driver* aktif atau semuanya Z | Z |
| Semua *driver* aktif bernilai 0 | 0 |
| Semua *driver* aktif bernilai 1 | 1 |
| Ada *driver* 0 dan *driver* 1 bersamaan | X |
| Ada *driver* X yang aktif | X |

Tabel 2.1 menunjukkan bahwa nilai net ditentukan oleh *driver* yang aktif, bukan oleh banyaknya *driver*. Net hanya bernilai 0 atau 1 bila seluruh *driver* aktif sepakat, bernilai Z bila tidak ada yang aktif, dan bernilai X bila terjadi konflik atau ketidakpastian. Dengan aturan ini, konflik antar-*driver* tidak diam-diam diselesaikan menjadi 0 atau 1.

*Tri-state buffer* adalah komponen yang meneruskan masukannya saat dihidupkan dan mengeluarkan Z saat dimatikan. Komponen ini memungkinkan beberapa sumber berbagi satu jalur data (bus) secara sah, selama hanya satu yang aktif pada satu waktu. Simulator seperti Logisim Evolution membedakan nilai *floating* dari *error* akibat konflik *driver*, dan menyediakan opsi untuk memperlakukan masukan yang tidak terhubung sebagai *error* (Logisim-evolution, n.d.). Pembedaan ini penting bagi pengguna, karena kesalahan struktur rangkaian dan ketidakpastian nilai adalah dua hal yang berbeda.

#### 2.1.3 Combinational Feedback dan Penyelesaian Stable State

*Combinational feedback* terjadi ketika keluaran sebuah gerbang kembali memengaruhi masukannya sendiri melalui jalur gerbang lain. Tidak semua *feedback* merupakan kesalahan. *Latch* SR yang disusun dari dua gerbang NAND atau NOR yang saling menyilang justru memakai *feedback* untuk menyimpan satu bit. Sebaliknya, inverter yang keluarannya dihubungkan ke masukannya sendiri tidak memiliki nilai stabil dan berosilasi. Perbedaan ini menuntut simulator mengenali kelompok operasi yang saling bergantung secara melingkar, yang dalam teori graf disebut *strongly connected component* (SCC), lalu menyelesaikannya secara iteratif sampai tidak ada nilai yang berubah. Iterasi perlu dibatasi dan osilasi dilaporkan, seperti halnya Logisim Evolution yang menjadikan batas iterasi osilasi sebagai kebijakan yang terlihat oleh pengguna (Logisim-evolution, n.d.). Hasil penyelesaian tidak boleh bergantung pada urutan komponen di *file* atau pada pilihan acak.

Pentingnya aturan yang eksplisit dipertegas oleh Lööw (2025), yang menganalisis secara formal semantik simulasi Verilog sintesis dan menunjukkan adanya ketidakkonsistenan pada definisi *scheduling* yang dapat berbenturan dengan praktik simulator. Pelajarannya adalah bahwa semantik sebaiknya ditulis sebagai aturan operasional yang konsisten dan dapat diuji, bukan sekadar menirukan istilah dari bahasa deskripsi perangkat keras. Standar IEEE 1800-2023 untuk SystemVerilog (IEEE, 2024) mencakup perilaku tingkat RTL dan tingkat gerbang beserta banyak fitur bahasa lain, sehingga lebih tepat dipakai sebagai rujukan semantik daripada spesifikasi yang harus diimplementasikan seluruhnya.

#### 2.1.4 Memori

Memori RAM dan ROM merupakan komponen sekuensial berkapasitas besar. ROM hanya dapat dibaca selama simulasi dan isinya berasal dari data inisialisasi. Pada RAM, perilaku yang sah bergantung pada konfigurasi *port*. Dokumentasi *memory library* Yosys, perangkat sintesis sumber terbuka, membedakan perilaku *read-during-write* (baca saat tulis) menjadi beberapa mode, yaitu mengembalikan isi lama (*old*), isi baru (*new*), tidak berubah (*no\_change*), dan tidak terdefinisi (*undefined*), serta mengatur transparansi tulis antar-*port* secara terpisah (YosysHQ, n.d.-a). Karena itu, simulator perlu menetapkan secara eksplisit kebijakan *read-during-write*, jenis baca (asinkron atau tersinkron), dan perilaku untuk alamat di luar jangkauan atau alamat yang tidak diketahui. Alamat yang tidak diketahui tidak boleh diam-diam dianggap sebagai alamat 0, melainkan menghasilkan nilai X pada hasil baca.

#### 2.1.5 Lebar Bus

Bus adalah sekelompok *wire* yang membawa beberapa bit sekaligus, misalnya 8, 32, atau 64 bit. Lebar (*width*) merupakan bagian dari kontrak koneksi, bukan sekadar urusan tampilan. Dua pin hanya dapat terhubung langsung bila lebarnya sama, dan *truncation* (pemotongan lebar) atau *extension* (perluasan lebar) tidak dilakukan diam-diam. *Zero-extension* (perluasan dengan nol) dan *sign-extension* (perluasan dengan bit tanda) adalah operasi yang berbeda. Untuk bus sampai 64 bit, satu *machine word* dapat menampung seluruh bit sehingga operasi *bitwise* berjalan sekaligus, dengan syarat semantik per bit tetap terjaga.

### 2.2 Teknik Simulasi Logika Digital

Setelah aturan semantik ditetapkan, pertanyaan berikutnya adalah bagaimana menjalankan aturan itu dengan cepat. Literatur menyediakan beberapa keluarga teknik, dan setiap teknik memiliki kekuatan serta biaya yang berbeda.

#### 2.2.1 Compiled Simulation dan Event-Driven Simulation

Gunes et al. (2005) membedakan dua mekanisme dasar simulator digital. Pada *compiled simulation*, *netlist*, yaitu daftar sambungan antarkomponen, diubah menjadi urutan instruksi yang merepresentasikan fungsi dan hubungan antarkomponen, dengan tabel nilai untuk *state* sinyal. Pada *event-driven simulation*, simulator memakai *event queue* dan daftar evaluasi untuk memproses hanya komponen yang inputnya berubah.

Bentuk paling sederhana dari pendekatan *compiled* adalah *levelized full-cycle execution*, yaitu eksekusi satu siklus penuh yang diurutkan menurut level. Pada tahap *compile*, gerbang kombinasional diurutkan menurut ketergantungannya sehingga setiap gerbang dievaluasi setelah semua inputnya siap. Pada setiap siklus, evaluator dijalankan menurut urutan itu. Elemen sekuensial menjadi batas antarsiklus, karena nilai berikutnya dihitung dari *state* saat ini dan seluruh *state* disimpan serentak pada batas *clock*. Kekuatan pendekatan ini ialah tidak diperlukannya *queue* dinamis, *execution plan* dibuat sekali, dan jalur eksekusi dapat berupa akses *array* dan operasi sederhana. Kelemahannya ialah gerbang tetap dievaluasi walaupun inputnya tidak berubah, perubahan struktur rangkaian menuntut pembangunan ulang, dan *combinational feedback* tidak dapat ditangani dengan pengurutan topologis biasa.

Pendekatan *event-driven* memiliki kekuatan yang berlawanan. Kerja simulator sebanding dengan aktivitas rangkaian, dan model ini wajar untuk *propagation delay* dan logika asinkron. Namun *queue*, pencatat waktu, alokasi memori, dan pembatalan *event* menambah biaya. Pada CPU sinkron yang mengaktifkan banyak jalur data setiap siklus, pencatatan *event* dapat lebih mahal daripada evaluasi langsung.

#### 2.2.2 Model Waktu

Gunes et al. (2005) juga membedakan model waktu *zero-delay* (tanpa penundaan), *unit-delay* (penundaan satu satuan), *nominal delay* (penundaan menurut nilai tipikal), dan *min/max delay* (penundaan batas minimum dan maksimum). Model yang lebih akurat membutuhkan biaya *scheduling* yang lebih besar. Pada model *zero-delay*, perubahan dianggap merambat seketika, sehingga hanya urutan ketergantungan yang penting. Bagi simulator yang tujuannya menjalankan CPU digital dan bukan memprediksi waktu fisik, simulasi fungsional berbasis siklus dengan model *zero-delay* merupakan titik awal yang paling masuk akal. Logika kombinasional diselesaikan dalam satu siklus, kemudian elemen penyimpan memperbarui nilainya pada batas *clock*.

#### 2.2.3 Pendekatan Hibrida dan Peran Tingkat Aktivitas

Wang dan Maurer (1990) memadukan kedua keluarga tersebut pada LECSIM. Simulator ini memakai model *zero-delay* dan *event scheduling* yang diurutkan menurut level untuk menghindari evaluasi yang tidak perlu, kemudian meng-*compile* *scheduler* pusat menjadi segmen *scheduling* lokal. Mereka melaporkan simulator ini berjalan sekitar 8 hingga 77 kali lebih cepat daripada simulator *event-driven* tradisional yang bersifat *interpreted* pada *benchmark* yang dipakai. Mereka juga mencatat bahwa *levelized compiled simulation* hanya kalah dari *event-driven simulation* bila aktivitas rangkaian sangat rendah, di bawah sekitar 1 persen, suatu keadaan yang menurut mereka jarang terjadi dalam praktik.

Maurer (2003) mengusulkan pengelompokan gerbang menjadi *fanout-free region* (FFR), yaitu kelompok gerbang yang jalur internalnya tidak bercabang ke luar kelompok, sehingga keluaran kelompok dapat dipantau sebagai satu unit. Jika keluaran sebuah FFR tidak berubah, FFR di hilirnya tidak perlu dievaluasi. Studi tersebut juga menunjukkan bahwa pilihan antara *event-driven* dan *levelized compiled simulation* sangat bergantung pada tingkat aktivitas, yakni *event-driven* cenderung unggul pada aktivitas rendah dan *compiled simulation* pada aktivitas tinggi.

Beamer dan Donofrio (2020) memberi peringatan penting. Walaupun banyak sinyal jarang berubah, simulator RTL modern sering tetap memakai *full-cycle evaluation*, karena biaya mendeteksi perubahan dan menjadwal ulang komputasi dapat melebihi biaya menghitung ulang. Mereka menelaah *coarse-grained activity tracking* (pelacakan aktivitas berbutir kasar) melalui partisi yang berisi puluhan hingga ratusan elemen dan *acyclic partitioning*. Kabiri Chimeh et al. (2012) membandingkan evaluasi sinkron *brute-force*, *event queue*, algoritma *marking* (penandaan komponen yang berubah) tanpa struktur data pusat, dan *lazy evaluation* pada CPU berurutan maupun GPGPU (general-purpose computing on GPU, yaitu pemakaian kartu grafis untuk komputasi umum). *Dirty marking* (penandaan partisi yang berubah) dapat menjadi kompromi antara pemindaian penuh dan *event queue* penuh, dengan syarat biaya *marking* lebih kecil daripada evaluasi yang dihemat. Rangkuman temuan ini memberi pesan yang konsisten. Tidak ada teknik yang otomatis unggul, sehingga keputusan harus ditentukan oleh aktivitas nyata *workload* dan diukur pada semantik yang sama.

#### 2.2.4 Teknik Optimasi Lanjutan

Selain *scheduling*, literatur mengenal beberapa teknik optimasi yang dapat ditumpuk di atas dasar yang benar. *Lazy evaluation* menghitung nilai hanya ketika dibutuhkan oleh konsumen atau keluaran yang diamati (Charlton et al., 1991). Teknik ini berguna untuk *logic cone* kombinasional murni, tetapi berisiko pada komponen yang menyimpan *state*, sebab *state* yang tidak sedang diamati tetap dapat memengaruhi siklus berikutnya. *Bit-parallel evaluation* memanfaatkan satu *machine word* untuk memproses banyak bit sekaligus, baik bit dari satu bus maupun banyak pola uji yang dikemas pada bit berbeda. Gupta dan Pingali (1995) mewakili logika Boolean pada FFR sebagai *binary decision diagram* (BDD), yaitu struktur graf untuk fungsi Boolean, lalu mengubahnya menjadi kode yang dapat dieksekusi, dan melaporkan peningkatan hingga 67 persen dibanding simulasi *oblivious* pada *benchmark* mereka. Ukuran BDD sensitif terhadap urutan variabel dan dapat membesar drastis, sehingga bukan dasar yang aman.

*Native code generation* meng-*compile* rangkaian menjadi *machine code* untuk menghilangkan pemilihan tipe gerbang saat berjalan. Au et al. (1991) membahas pembangkitan simulasi *compiled* secara otomatis melalui *program specialization*, dan Huff dan Hanrahan (2018) meng-*compile* versi yang dikhususkan setelah konfigurasi arsitektur dimuat. Eksekusi paralel pada CPU atau GPU memanfaatkan gerbang yang saling bebas pada level atau partisi yang sama. Bailey et al. (1994) menunjukkan bahwa kinerja simulasi logika paralel dipengaruhi algoritma sinkronisasi, struktur rangkaian, granularitas waktu, arsitektur target, dan partisi. Banyak rangkaian memiliki level yang sempit sehingga biaya *scheduling* dan sinkronisasi dapat melebihi kerja gerbang itu sendiri. Ringkasan teknik-teknik tersebut disajikan pada Tabel 2.2.

**Tabel 2.2** Ringkasan teknik simulasi logika digital

| Teknik | Prinsip | Kekuatan | Risiko atau biaya |
| --- | --- | --- | --- |
| *Levelized full-cycle* | Semua gerbang dievaluasi menurut urutan ketergantungan | Sederhana dan dapat diprediksi | Gerbang yang tidak berubah tetap dihitung |
| *Event-driven* | Hanya komponen terdampak yang dihitung | Kerja sebanding aktivitas | Biaya *queue* dan *scheduling* |
| *Hybrid levelized* | *Event* digabung dengan jadwal *compiled* | Mengurangi *event* dan biaya *scheduler* | *Compiler* lebih kompleks |
| *Partition marking* | Partisi yang berubah ditandai *dirty* | *Activity tracking* berbutir kasar | Ukuran partisi perlu disetel |
| *Fanout-free region* | Gerbang dikelompokkan menjadi satu unit | Mengurangi *event* pada *fan-out* | Ukuran FFR perlu disetel |
| *Lazy evaluation* | Nilai dihitung saat dibutuhkan | Menghindari kerja yang tidak perlu | Sulit untuk logika yang menyimpan *state* |
| *Bit-parallel* | Satu *word* memuat banyak bit | Operasi bus lebih cepat | *Encoding* logika multinilai lebih rumit |
| *BDD-based* | Fungsi Boolean dipadatkan menjadi graf | Evaluasi Boolean lebih ringkas | Ukuran BDD dapat membesar |
| *Machine code* atau JIT | Rangkaian di-*compile* ke *native code* | Mengurangi biaya pemilihan operasi | Portabilitas, *debugging*, biaya *compile* |
| *Parallel* CPU atau GPU | Gerbang saling bebas dihitung bersamaan | Memanfaatkan banyak inti | Biaya sinkronisasi dan memori |

Tabel 2.2 menunjukkan bahwa setiap teknik menukar satu jenis biaya dengan jenis biaya lain. Teknik yang mengurangi pekerjaan evaluasi, seperti *event-driven*, *partition marking*, dan *lazy evaluation*, menambah biaya pencatatan atau kerumitan *state*. Teknik yang menurunkan biaya per operasi, seperti *bit-parallel*, *machine code*, dan eksekusi paralel, menambah kerumitan implementasi atau sinkronisasi. Karena itu *levelized full-cycle* yang sederhana dan dapat diprediksi dipilih sebagai titik awal DLF, sedangkan teknik lain dipertimbangkan bila hasil pengukuran membenarkannya.

### 2.3 Arsitektur Compiler dan Runtime Simulator

Teknik simulasi hanya berguna bila dapat ditanamkan pada arsitektur perangkat lunak yang jelas. Subbab ini membahas bagaimana rangkaian yang digambar pengguna diubah menjadi data yang dapat dijalankan dengan cepat, dan bagaimana hasilnya sampai ke layar tanpa menghambat simulasi. Subbab 2.3.1, 2.3.3, dan 2.3.6 merujuk pada dokumentasi Verilator, LLVM, dan Godot, sedangkan 2.3.2, 2.3.4, dan 2.3.5 merupakan rancangan penulis yang disusun dari praktik tersebut dan dijabarkan lebih lanjut pada Bab III.

#### 2.3.1 Pemisahan Tiga Representasi

Pola yang berulang pada simulator berkinerja tinggi adalah memisahkan tiga representasi. Yang pertama adalah representasi penyuntingan (*authoring*), yaitu struktur yang nyaman untuk disunting, disimpan, dan ditampilkan, seperti hierarki *chip*, *instance* komponen, pin, *wire*, posisi visual, dan label. Yang kedua adalah representasi *compiled*, yaitu struktur ringkas yang telah divalidasi dan dioptimalkan untuk eksekusi tanpa mempertahankan graf objek *editor*. Yang ketiga adalah *runtime state*, yaitu nilai sinyal, register, memori, dan masukan yang berubah selama simulasi, yang sebaiknya terpisah dari *execution plan* yang relatif tetap.

Verilator menunjukkan pola ini. Dokumentasinya menyebut Verilator sebagai *compiler* yang mengolah desain melalui sejumlah tahap lalu menghasilkan model C++ yang dievaluasi melalui lapisan pembungkus (*wrapper*), bukan simulator tradisional yang menafsirkan HDL secara langsung (Verilator, n.d.-b). Pemisahan serupa terlihat pada Digital Logic Sim, yang memisahkan deskripsi *chip* (`ChipDescription`) dari *instance* *runtime* (`SimChip`) pada *source code*-nya (Lague, n.d.-b).

#### 2.3.2 Tahapan Compile

Alur dari dokumen rangkaian sampai *instance* simulasi dapat dipecah menjadi sembilan tahap berurutan berikut. Urutan ini adalah rancangan DLF yang disusun dari praktik simulator pada literatur di atas, dan keputusan rinci per tahap dibahas pada subbab 3.4.1.

1. **Snapshot.** Saat pengguna memulai simulasi, *editor* menghasilkan salinan dokumen yang tidak dapat diubah (*immutable*). Simulator tidak membaca objek *editor* yang masih dapat berubah.
2. **Structural validation.** Pemeriksaan pin wajib, kecocokan lebar bus, definisi komponen yang hilang, dan koneksi tidak valid, dengan *error* yang menyertakan lokasi visual.
3. **Hierarchy elaboration.** Setiap *instance* *chip* dikembangkan menjadi graf *instance* konkret dan parameternya diselesaikan. Rekursi *chip* yang tak terbatas dideteksi dan ditolak.
4. **Connectivity construction.** Setiap pin dipetakan ke *slot* sinyal *runtime*, dan beberapa segmen *wire* yang terhubung secara listrik dipetakan ke satu net. *Wire* visual tidak perlu menjadi objek *runtime*.
5. **Semantic analysis.** Aturan nilai sinyal, resolusi *driver*, dan batas sekuensial diterapkan. Elemen penyimpan seperti register, *latch*, dan memori diidentifikasi, keluarannya menjadi sumber nilai saat ini, dan masukannya menjadi target perhitungan *next state*.
6. **Combinational dependency analysis.** Graf ketergantungan antaroperasi dibangun dan siklus dideteksi. Wilayah tanpa siklus diurutkan secara topologis atau berlevel (*levelization*), sedangkan siklus (SCC) dilaporkan beserta jalurnya dan tidak diselesaikan dengan urutan sembarang.
7. **IR lowering.** Graf komponen diturunkan menjadi catatan operasi sederhana, disertai *constant folding* dan *dead logic elimination* selama pengamatan sinyal tidak terganggu.
8. **Runtime layout allocation.** *Array* sinyal, *array* *state*, blok memori, serta tabel masukan dan keluaran dialokasikan, sehingga jalur eksekusi tidak perlu mencari berdasarkan nama atau identitas objek.
9. **Artifact finalization.** Hasilnya memuat *execution plan*, deskripsi *layout*, *source map*, diagnostik, versi *compiler*, profil semantik, dan *fingerprint* dokumen.

#### 2.3.3 Representasi Runtime dan IR

IR atau *intermediate representation* adalah bentuk antara yang lebih sederhana daripada rangkaian visual tetapi belum berupa *machine code*. Setiap catatan operasi memuat kode operasi (*opcode*), *slot* tujuan, *slot* sumber, lebar bit, dan penanda tambahan. Contohnya, rangkaian yang memasukkan A dan B ke gerbang AND lalu hasilnya ke XOR bersama C menjadi dua catatan operasi yang merujuk indeks *array* sinyal. Catatan disimpan pada *array* yang berurutan di memori, bukan satu objek per gerbang, sehingga akses lebih ramah terhadap *cache* prosesor. Empat pilihan representasi *runtime* dibandingkan pada Tabel 2.3.

**Tabel 2.3** Perbandingan representasi runtime

| Pendekatan | Mekanisme | Kelebihan | Kekurangan |
| --- | --- | --- | --- |
| *Object-graph interpreter* | Menelusuri objek komponen dan pin | Mudah dibuat dan di-*debug* | *Pointer chasing*, alokasi, dan penelusuran hierarki |
| *Compact bytecode* | *Array* instruksi dengan *opcode* dan indeks *integer* | Portabel, *compile* cepat, mudah di-*profile* | Pemilihan *opcode* tetap terjadi pada setiap operasi |
| *Threaded code* | Instruksi menunjuk fungsi evaluator | Mengurangi *dispatch* berbasis *switch* | *Indirect branch*, belum tentu lebih cepat |
| *Machine code* atau JIT | Rangkaian diterjemahkan menjadi *machine code* | Menghilangkan *dispatch* *interpreter* | Latensi *compile*, *dependency*, portabilitas, keamanan |

Tabel 2.3 memperlihatkan bahwa semakin dekat sebuah representasi ke *machine code*, semakin kecil biaya *dispatch* per operasi, tetapi semakin besar biaya *compile*, *dependency*, dan risiko portabilitas. *Compact bytecode* berada di tengah, yaitu cukup ringkas untuk dioptimalkan dan tetap mudah di-*debug*, sehingga dipilih sebagai *baseline* DLF (lihat Tabel 2.5).

*Code generation* ke *machine code* juga bukan jaminan lebih cepat. Dokumentasi Verilator menyebut ukuran kode yang dihasilkan dan *instruction cache* sebagai faktor penting, serta menunjukkan adanya pertukaran antara optimasi jalur cepat dan waktu *compile* (Verilator, n.d.-c). Untuk JIT di dalam proses aplikasi, ORC dari proyek LLVM menyediakan lapisan optimasi, *lazy compilation*, dan *concurrent compilation*, tetapi membawa *dependency* besar dan integrasi yang kompleks (LLVM Project, n.d.-a).

Penyimpanan sinyal turut menentukan kecepatan. Satu nilai *boolean* per pin terlalu boros untuk bus besar. Untuk bus sampai 64 bit, satu *integer* seukuran *machine word* dapat menyimpan seluruh nilai, dan operasi *bitwise* native dapat dipakai sekaligus. *Compiler* harus mengetahui lebar setiap sinyal dan menerapkan *width mask* agar bit di luar bus tidak bocor.

#### 2.3.4 Sequential State dan Siklus Eksekusi

Untuk model berbasis siklus, *runtime* membedakan *current state* dan *next state*. Satu siklus konseptual terdiri atas pembaruan masukan *runtime*, penyediaan keluaran elemen penyimpan sebagai sumber, evaluasi *execution plan* kombinasional, penghitungan masukan D atau *next state*, penyimpanan serentak seluruh *next state* pada batas *clock*, lalu pembaruan keluaran periferal dan *probe*. Register tidak boleh diperbarui satu per satu selama evaluasi kombinasional berlangsung, karena hasilnya akan bergantung pada urutan operasi.

#### 2.3.5 Hierarki, Source Map, dan Traceability

Hierarki *chip* adalah bagian penting pengalaman pengguna, tetapi tidak harus dieksekusi secara rekursif. Praktik yang lazim adalah menyimpan hierarki penuh di *editor*, mengelaborasi setiap *instance* saat *compile*, dan menjalankan versi yang di-*flatten* atau dikhususkan. *Source map* mempertahankan jalur hierarki, misalnya CPU/ALU/Adder/AND\_12, sehingga *debugger* dapat menyorot gerbang dan pin visual dari sinyal *runtime*. Setiap *instance* tetap memiliki *state* dan *slot* sinyalnya sendiri meskipun berasal dari definisi *chip* yang sama. Risikonya, rangkaian besar dapat menghasilkan IR yang sangat besar, sehingga *flattening* tidak berarti seluruh gerbang digabung menjadi satu fungsi raksasa. Verilator juga memetakan hasil *profiling* kode yang dihasilkan kembali ke sumber desain (Verilator, n.d.-c), prinsip *traceability* yang relevan bagi simulator visual.

#### 2.3.6 Simulation Worker dan Sinkronisasi dengan UI

Simulasi yang dijalankan di *thread* UI akan terbatas oleh kecepatan gambar. Karena itu, *simulation worker*, yaitu *thread* terpisah yang menjalankan simulasi, memiliki *runtime state* yang dapat berubah dan menjalankan sekelompok siklus dalam satu putaran, bukan satu siklus per *frame*. UI mengirim perintah seperti *run*, *pause*, *step*, *reset*, dan *stop*, serta masukan *runtime* seperti *keyboard* atau tombol, yang diterapkan pada batas yang jelas. Hasil dikirim sebagai *snapshot* yang tidak berubah atau himpunan keluaran yang berubah. *Double buffer* atau *ring buffer* dipakai agar *worker* dan *renderer* tidak membaca dan menulis *buffer* yang sama bersamaan. Bila UI lambat, *worker* tidak menunggu. *Snapshot* dapat dilewati atau digabung, sedangkan *trace* yang perlu lengkap disimpan melalui jalur terpisah. Dokumentasi Godot Engine menyatakan bahwa *scene tree* aktif tidak aman untuk banyak *thread* (Godot Engine, n.d.), sehingga *worker* tidak boleh memanipulasi objek UI secara langsung. Pada pola ini, merender 60 *frame* per detik tidak berarti simulasi hanya berjalan 60 siklus per *frame*, karena *worker* dapat menjalankan ribuan siklus atau lebih di antara dua pembaruan layar.

*Observability* memiliki biaya. Merekam seluruh sinyal pada setiap siklus menghabiskan *memory bandwidth* dan menurunkan *throughput*, sehingga mode *debug* dan mode cepat memerlukan kebijakan pengamatan yang berbeda, dengan pemilihan *probe* dan *trace sampling* yang eksplisit.

### 2.4 Verifikasi Kebenaran dan Pengukuran Kinerja

Simulator yang cepat tetapi salah tidak berguna, dan simulator yang benar tetapi tidak terukur tidak dapat dibandingkan. Karena itu evaluasi simulator memisahkan dua pertanyaan. Pertanyaan pertama ialah apakah *compiler* dan *runtime* mempertahankan semantik pada semua rangkaian dan masukan yang diuji. Pertanyaan kedua ialah berapa biaya *compile*, eksekusi, memori, dan presentasi pada *workload* yang representatif. Kebenaran menjadi gerbang pertama, sehingga pengukuran kinerja hanya bermakna setelah *correctness test* untuk *workload* terkait lulus.

#### 2.4.1 Metode Verifikasi Kebenaran

Tidak ada satu metode pengujian yang cukup, sehingga beberapa metode dipakai bersama karena saling menutup kelemahan.

- **Golden test.** Hasil yang telah diperiksa disimpan sebagai keluaran yang diharapkan, lalu dijadikan *regression test* agar perubahan kode tidak diam-diam mengubah perilaku.
- **Differential testing.** Rangkaian dan urutan masukan yang sama dijalankan pada *reference evaluator* yang sengaja sederhana dan pada *runtime* yang dioptimalkan, lalu keluaran, *state* elemen penyimpan, dan memori dibandingkan. Setiap perbedaan, sesuai gagasan McKeeman (1998), menjadi kandidat *bug*.
- **Property-based testing.** Masukan dan *state* acak dibangkitkan untuk memeriksa sifat yang harus selalu benar (Claessen & Hughes, 2000), misalnya hasil *adder* atau register yang menahan nilai.
- **Metamorphic testing.** Rangkaian diubah dengan cara yang seharusnya tidak memengaruhi perilaku, misalnya menggeser posisi visual, mengganti label, atau mengubah urutan penyimpanan komponen, lalu hasilnya dipastikan tetap sama (Chen et al., 2018).
- **Fuzzing.** Gagasan ini berawal dari penggunaan masukan acak untuk menemukan kegagalan program (Miller et al., 1990). Pada DLF, rangkaian kecil yang valid dibangkitkan secara acak lalu evaluator dibandingkan, dengan *random seed* dan *minimal failing case* disimpan.
- **Ekuivalensi formal (opsional).** Untuk subset rangkaian yang dapat diekspor ke representasi HDL atau Boolean, perangkat seperti Yosys dapat memeriksa ekuivalensi transformasi (YosysHQ, n.d.-b). Metode ini hanya membuktikan model dan asumsi yang dimasukkan, sehingga tidak menggantikan uji simulator secara keseluruhan.

*Differential testing* memiliki kelemahan. Bila *reference evaluator* dan *runtime* kandidat memakai kode primitif atau *resolver* yang sama, kesalahan bersama tidak terdeteksi. Karena itu *golden test* dan *property-based testing* dipakai sebagai pelengkap, dan implementasi referensi dibuat independen sejauh memungkinkan.

#### 2.4.2 Metrik dan Protokol Pengukuran

Kinerja dilaporkan per aspek, bukan sebagai satu skor gabungan. Metrik utamanya ialah waktu *compile* total dan per tahap, *cycles per second* (CPS), waktu per satu juta siklus, penggunaan memori puncak, *frame rate* UI, latensi dari perintah hingga tampil di layar, dan FPS aplikasi. CPS dihitung dari jumlah siklus yang disimulasikan dibagi *wall-clock time* (waktu nyata yang berlalu) simulasi.

```latex
\mathrm{CPS} = \frac{\text{siklus tersimulasi}}{\text{wall-clock time simulasi (detik)}}
```

CPS harus menyatakan apakah waktu *compile*, pemulaian *worker*, pemrosesan perintah, *trace recording*, dan *rendering* UI termasuk atau dikecualikan. Karena itu dilaporkan dua angka, yaitu CPS inti tanpa presentasi UI dan *throughput* ujung ke ujung dengan jalur aplikasi yang relevan. Pengukuran memori puncak juga tidak boleh disamakan dengan ukuran ruang alamat virtual atau *heap* saja. Beyer et al. (2019) membahas persyaratan pengukuran yang andal, termasuk *wall-clock time*, waktu CPU, dan memori puncak, serta pengelolaan proses anak.

Protokol pengukuran mengikuti praktik umum. Google Benchmark menyediakan *warm-up* (putaran pemanasan sebelum pengukuran), pengulangan, dan agregasi statistik seperti rata-rata, median, simpangan baku, dan koefisien variasi (Google, n.d.), sedangkan panduan LLVM menekankan pengulangan dan pengurangan sumber *noise* (LLVM Project, n.d.-b). Praktik yang dirangkum untuk proyek ini adalah memakai *release build*, mencatat spesifikasi perangkat keras dan konfigurasi, menjalankan *warm-up* yang tidak dihitung, mengulang pengukuran utama sedikitnya 10 kali, melaporkan median beserta sebaran, memakai *high-resolution monotonic timer*, dan menyimpan hasil mentah bersama *configuration manifest*. Peningkatan yang selisihnya tidak melampaui *noise* pengukuran tidak diklaim sebagai peningkatan.

#### 2.4.3 Workload dan Baseline

*Workload* uji dibagi menjadi dua kelompok. Kelompok diagnostik, yaitu logika primitif, penyebaran *fanout*, jalur data aritmetika, bus, rangkaian sekuensial, memori, *feedback*, rangkaian hierarkis, dan replikasi skala sintetis, mengisolasi satu aspek sehingga penyebab perubahan kinerja dapat dikenali. Kelompok sistem, yaitu CPU sederhana, tampilan interaktif, dan program demonstrasi seperti Snake atau Pong, menunjukkan apakah optimasi tingkat rendah benar-benar membantu tujuan produk. *Benchmark circuit* standar tersedia pada ISCAS'85 untuk rangkaian kombinasional dan ISCAS'89 untuk rangkaian sekuensial, yaitu kumpulan rangkaian uji dari International Symposium on Circuits and Systems tahun 1985 dan 1989 yang lazim dipakai sebagai tolok ukur, dan tersedia dalam beberapa format file seperti BENCH, BLIF, EDIF, dan Verilog (Czech Technical University in Prague, n.d.). Rangkaian tersebut tidak otomatis kompatibel karena inisialisasi sekuensial, pengaturan *clock*, nilai tidak diketahui, dan pemetaan primitif harus diperiksa lebih dulu.

*Baseline* dibuat berjenjang karena tidak ada satu simulator eksternal yang adil untuk semua aspek. *Baseline* tersebut mencakup *reference evaluator* yang transparan, *runtime* awal yang sederhana, varian *runtime* kandidat, simulator eksternal bila *workload* dan semantiknya dapat disejajarkan, dan aplikasi DLF ujung ke ujung. Verilator memakai RTLMeter untuk membandingkan kecepatan simulasi, konsumsi memori, dan waktu penerjemahan (*translation time*), serta memperingatkan bahwa mesin yang dihosting dapat memiliki variasi tinggi (Verilator, n.d.-d). Hasil pembandingan dengan simulator HDL semacam itu hanya bersifat kontekstual.

#### 2.4.4 Ancaman terhadap Validitas

Beberapa ancaman perlu diwaspadai. Perbandingan eksternal tidak sah bila aturan *clock*, X dan Z, memori, atau inisialisasi berbeda. Rangkaian sintetis dapat menguntungkan satu algoritma secara tidak realistis, sehingga mikro-*benchmark* perlu disandingkan dengan *workload* CPU dan aplikasi. Keluaran harus dikonsumsi agar *compiler* tidak menghapus pekerjaan yang diukur. *Rendering* dan *trace recording* dapat menjadi *bottleneck* yang keliru dianggap *bottleneck* simulasi. Laptop dapat menurunkan frekuensi saat beban berkelanjutan, sehingga mode daya dan kondisi termal perlu dicatat. Optimasi pada satu rangkaian tidak boleh diperlakukan sebagai peningkatan umum tanpa *test suite* yang lebih luas.

### 2.5 Tinjauan Simulator Terkait

Subbab ini mengkaji lima simulator visual yang paling dekat dengan tujuan proyek. Kajian memakai dokumentasi publik dan, bila tersedia, inspeksi *source code*. Tiga produk tidak menyediakan *source code* *engine* simulasinya, sehingga analisisnya dibatasi pada fitur yang terlihat dan klaim pengembang, dan klaim tersebut diberi label sesuai kualitas buktinya.

#### 2.5.1 Digital Logic Sim

Digital Logic Sim (DLS) karya Sebastian Lague menjadi acuan utama untuk filosofi interaksi dan hierarki *chip*. Pengguna mulai dari gerbang sederhana, lalu membuat *chip* kustom yang dapat dipakai di dalam *chip* lain untuk membangun kompleksitas bertingkat (Lague, n.d.-a). Inspeksi *source code* pada *branch* utama repositori, yang diakses pada 5 November 2026, menunjukkan beberapa hal yang relevan (Lague, n.d.-b). Pertama, deskripsi *chip* (`ChipDescription`) dipisahkan dari *instance* *runtime* (`SimChip`), sehingga model penyuntingan tidak harus menjadi struktur data yang dipakai jalur eksekusi. Kedua, langkah simulasi memproses komponen bawaan melalui percabangan `switch` menurut tipe, sedangkan *chip* kustom diproses secara rekursif. Ketiga, *scheduler*-nya bukan *event queue* generik, melainkan penentuan kesiapan *chip* berdasarkan apakah semua input yang terhubung telah diterima pada *frame* tersebut. Urutan hasilnya dipakai ulang pada *frame* berikutnya, dengan kemungkinan pertukaran urutan tertentu secara acak untuk memvariasikan hasil *race condition*, yaitu keadaan ketika hasil bergantung pada urutan pemrosesan yang tidak pasti. Keempat, bila tidak ada *chip* yang siap, implementasi memilih salah satu *chip* yang tersisa secara acak sebagai cara menyelesaikan *feedback*. Kelima, kode mencatat gagasan optimasi yang belum dikerjakan, seperti *lookup table* untuk *chip* kombinasional dan melewati *chip* yang inputnya tidak berubah. Keenam, simulasi berjalan pada *thread* terpisah, dan perubahan dari UI dikirim melalui *command queue*. Inspeksi ini tidak membuktikan kinerja relatif DLS terhadap simulator lain, dan tidak semua fitur dapat disimpulkan dari beberapa *file* *runtime* yang diperiksa.

#### 2.5.2 Virtual Circuit Board

Virtual Circuit Board (VCB) menjadi acuan untuk alur Edit dan Simulate. Halaman produknya menjelaskan bahwa penyuntingan dan simulasi dipisahkan. Pada mode penyuntingan pengguna menggambar rangkaian dan menulis program *assembly*, sedangkan pada mode simulasi rangkaian berjalan dan instruksi dapat diamati. Pengembang menyebut *engine*-nya sebagai simulasi *tick-based* berkinerja tinggi, yaitu sinyal merambat antarkomponen satu *tick* pada satu waktu, dan mengklaim frekuensi hingga jutaan *update* per detik (*Virtual Circuit Board*, n.d.). Angka itu merupakan klaim produk, bukan hasil *benchmark* independen yang telah dinormalisasi. VCB bersifat tertutup, sehingga tidak dapat disimpulkan bahwa ia memakai *event-driven scheduling*, JIT, atau *machine code generation*. Yang dapat diamati hanyalah perilaku produk dan catatan perubahan yang dipublikasikan pengembang.

#### 2.5.3 Logisim Evolution

Logisim Evolution merupakan simulator rangkaian digital klasik yang lebih dekat ke alat desain dan pendidikan. Inspeksi kodenya menunjukkan simulasi berjalan pada *thread* terpisah dengan pengaturan *clock* otomatis, *clock* manual, *single step*, dan *reset*. Komponen `Propagator` memiliki *event queue* yang menyimpan waktu, lokasi, komponen penyebab, dan nilai yang di-*propagate*. *Queue* ini dapat berbentuk beberapa implementasi, termasuk `PriorityQueue` Java dan *splay queue*, yang menunjukkan bahwa struktur *queue* sendiri merupakan pertukaran implementasi yang dapat dievaluasi. *Propagation* diproses sampai *queue* kosong atau batas osilasi terlampaui. Pemicuan *clock* dipisahkan dari penyelesaian *combinational propagation*, sehingga perubahan *clock* tidak bercampur dengan penyelesaian logika (Logisim-evolution, n.d.). Simulator ini dirancang untuk simulasi interaktif dengan semantik *propagation* yang eksplisit, bukan untuk *throughput* CPU kustom yang sangat tinggi. Karena itu ia menjadi acuan semantik dan *debugging*, bukan pembanding kinerja langsung.

#### 2.5.4 Turing Complete

Turing Complete menekankan perjalanan dari gerbang logika menuju komponen, arsitektur, dan bahasa *assembly* (*Turing Complete*, n.d.). Produk ini lebih berorientasi pada *puzzle* dan pembelajaran bertahap daripada *editor* *sandbox* untuk simulasi berkinerja tinggi. Profil publik menampilkan statistik seperti jumlah gerbang, penundaan, dan siklus untuk setiap solusi. Statistik tersebut merupakan properti rangkaian, bukan ukuran kecepatan simulator, dan keduanya menjawab pertanyaan yang berbeda. *Engine* internalnya tidak tersedia untuk diperiksa.

#### 2.5.5 Tiny Logic

Tiny Logic adalah produk yang paling relevan dengan tujuan membangun komputer dari rangkaian lalu menjalankan perangkat lunak yang kompleks. Produk dalam pengembangan ini dipublikasikan pada Oktober 2026. Halaman produknya menyatakan bahwa simulatornya bekerja hingga level transistor dan perilaku listrik, mendukung bus hingga 64 bit, memori, *keyboard*, osiloskop, dan layar piksel, serta meng-*compile* rangkaian menjadi *machine code* saat pengguna bermain (whoismuzzy, 2026). Repositori demonstrasi TINY-32 memuat *file* rangkaian, program DOOM, dan *source port*, dan menyebut bahwa pada *run* pertama papan di-*compile* selama beberapa detik sebelum hasilnya disimpan sehingga *startup* berikutnya cepat (PlangoDev, n.d.). *Source code* simulatornya tidak tersedia, sehingga repositori tersebut berisi rangkaian dan perangkat lunak target, bukan *source code* simulator. Karena produk ini masih sangat baru, klaim fitur dan kinerjanya perlu diperiksa ulang pada saat laporan akhir disusun.

#### 2.5.6 Ringkasan Perbandingan

Tabel 2.4 merangkum perbandingan kelima simulator menurut fokus produk, pemisahan mode Edit dan Simulate, strategi *runtime* yang diketahui, ketersediaan *source code*, level simulasi, dan nilai acuannya bagi DLF.

**Tabel 2.4** Perbandingan simulator terkait

| Aspek | Digital Logic Sim | VCB | Logisim Evolution | Turing Complete | Tiny Logic |
| --- | --- | --- | --- | --- | --- |
| Fokus | *Sandbox* visual dan *chip* hierarkis | *Sandbox* berbasis *tick* | Desain dan pendidikan logika digital | *Puzzle* dan belajar arsitektur | *Sandbox* tingkat transistor |
| Mode Edit dan Simulate terpisah | Tidak, simulasi berjalan sambil menyunting | Ya | *Propagation* otomatis atau manual | Mengikuti interaksi gim | Belum cukup informasi publik |
| Strategi *runtime* yang diketahui | Penelusuran hierarki rekursif dan urutan input siap | Berbasis *tick*, detail tertutup | *Event queue* | Tidak diverifikasi | *Compile* ke *machine code* menurut pengembang |
| *Source code* *engine* simulasi publik | Ya | Tidak ditemukan | Ya | Tidak ditemukan | Tidak ditemukan |
| Level simulasi | Gerbang dan komponen digital | Digital berbasis *tick* | Gerbang dan komponen digital | Digital | Transistor dan listrik |
| Nilai acuan bagi DLF | Pengalaman pengguna, hierarki, pemisahan *editor* dan *runtime* | Alur kerja Edit/Simulate | Semantik *event* dan *debugging* | Abstraksi belajar bertahap | Pesaing terdekat pada kinerja dan eksekusi CPU |

Tabel 2.4 menunjukkan pola yang sama pada kelima simulator. Hanya Digital Logic Sim dan Logisim Evolution yang *source code*-nya dapat diperiksa, sedangkan VCB dan Tiny Logic, yang paling menonjolkan kinerja, bersifat tertutup sehingga strategi *runtime*-nya tidak dapat diverifikasi. Turing Complete berorientasi pada pembelajaran. Karena itu nilai acuan yang diambil DLF dari tiap produk bersifat arsitektural dan pengalaman pengguna, bukan angka kinerja.

### 2.6 Posisi Proyek terhadap Kajian Pustaka

Kajian teori, teknik, arsitektur, evaluasi, dan simulator terkait memberi dasar bagi keputusan awal proyek. Tabel 2.5 memetakan temuan pustaka ke keputusan rancangan DLF. Keputusan tersebut bersifat *baseline*, artinya titik awal yang masuk akal menurut bukti yang ada dan masih harus diuji melalui *prototype* dan pengukuran.

**Tabel 2.5** Pemetaan temuan pustaka ke keputusan awal DLF

| Temuan pustaka | Sumber | Keputusan awal DLF |
| --- | --- | --- |
| Tidak ada teknik *scheduling* yang unggul di semua *workload* | Maurer (2003); Beamer & Donofrio (2020); Wang & Maurer (1990) | *Execution plan* statis *levelized* untuk wilayah kombinasional tanpa siklus, dan pendekatan *event-driven* hanya dipertimbangkan bila hasil ukur membenarkannya |
| *Compiler* memisahkan model sumber, hasil *compile*, dan *runtime state* | Verilator (n.d.-b); Lague (n.d.-b) | Dokumen rangkaian, artefak hasil *compile*, dan *runtime state* dipisahkan, dengan *compile* dijalankan saat Simulate |
| Hasil simulasi tidak boleh bergantung pada urutan acak | Lague (n.d.-b); Lööw (2025) | Semantik empat nilai yang deterministik dan terdokumentasi, serta diagnostik untuk rangkaian yang belum didukung |
| *Machine code* tidak otomatis lebih cepat | Verilator (n.d.-c); LLVM Project (n.d.-a) | *Bytecode* ringkas sebagai *baseline*, sedangkan *machine code generation* ditunda sampai pengukuran membuktikan kebutuhannya |
| UI tidak boleh menahan simulasi | Lague (n.d.-b); Logisim-evolution (n.d.); Godot Engine (n.d.) | *Simulation worker* terpisah yang memiliki *runtime state* dan mengirim *snapshot* ke UI |
| Klaim kinerja sulit dibandingkan tanpa protokol | Beyer et al. (2019); Google (n.d.) | Protokol *benchmark* yang dapat diulang, dengan CPS inti dan FPS aplikasi dilaporkan terpisah |

Dari pemetaan tersebut tampak kesenjangan yang dituju proyek ini. Teknik simulasi logika telah dibahas luas dalam literatur simulator HDL. Penerapannya pada simulator visual untuk membangun komputer, dengan semantik deterministik yang terdokumentasi dan *benchmark* yang terbuka, belum ditemukan pada produk yang dikaji dalam bab ini. Pernyataan ini terbatas pada produk dan sumber yang ditinjau, dan bukan klaim bahwa tidak ada karya lain.

DLF diposisikan pada kesenjangan tersebut. Proyek ini tidak bersaing dalam kedalaman fisik dengan simulator tingkat transistor, melainkan menggabungkan pengalaman penyuntingan visual seperti pada Digital Logic Sim (dengan hierarki komponen buatan pengguna sebagai target penguatan) dan alur Edit dan Simulate seperti pada VCB, ditopang oleh tahap *compile* yang eksplisit, semantik yang dapat diuji, dan evaluasi yang dapat diulang. Seluruh keputusan di atas diperlakukan sebagai asumsi rancangan. Target angka kinerja belum ditetapkan sebelum *workload* dasar dan hasil *prototype* tersedia, sehingga klaim akhir hanya dibuat sebatas bukti yang berhasil dikumpulkan.

## Referensi Sementara Bab II

Au, W. Y., Weise, D., & Seligman, S. (1991). Automatic generation of compiled simulations through program specialization. In *Proceedings of the 28th ACM/IEEE Design Automation Conference* (pp. 205–210). ACM. https://doi.org/10.1145/127601.127665

Bailey, M. L., Briner, J. V., & Chamberlain, R. D. (1994). Parallel logic simulation of VLSI systems. *ACM Computing Surveys, 26*(3), 255–294. https://doi.org/10.1145/185403.185424

Beamer, S., & Donofrio, D. (2020). Efficiently exploiting low activity factors to accelerate RTL simulation. In *2020 57th ACM/IEEE Design Automation Conference (DAC)* (pp. 1–6). IEEE. https://doi.org/10.1109/DAC18072.2020.9218632

Beyer, D., Löwe, S., & Wendler, P. (2019). Reliable benchmarking: Requirements and solutions. *International Journal on Software Tools for Technology Transfer, 21*(1), 1–29. https://doi.org/10.1007/s10009-017-0469-y

Charlton, C. C., Jackson, D., & Leng, P. H. (1991). Lazy simulation of digital logic. *Computer-Aided Design, 23*(7), 506–513. https://doi.org/10.1016/0010-4485(91)90049-3

Chen, T. Y., Kuo, F.-C., Liu, H., Poon, P.-L., Towey, D., Tse, T. H., & Zhou, Z. Q. (2018). Metamorphic testing: A review of challenges and opportunities. *ACM Computing Surveys, 51*(1), Article 4. https://doi.org/10.1145/3143561

Claessen, K., & Hughes, J. (2000). QuickCheck: A lightweight tool for random testing of Haskell programs. In *Proceedings of the Fifth ACM SIGPLAN International Conference on Functional Programming* (pp. 268–279). ACM. https://doi.org/10.1145/351240.351266

Czech Technical University in Prague. (n.d.). *Collection of digital design benchmarks*. https://ddd.fit.cvut.cz/www/prj/Benchmarks/index.php?page=download

Godot Engine. (n.d.). *Thread-safe APIs*. In *Godot documentation*. https://docs.godotengine.org/en/stable/tutorials/performance/thread\_safe\_apis.html

Google. (n.d.). *Benchmark user guide*. GitHub. https://github.com/google/benchmark/blob/main/docs/user\_guide.md

Gunes, M., Thornton, M. A., Kocan, F., & Szygenda, S. A. (2005). A survey and comparison of digital logic simulators. In *2005 48th Midwest Symposium on Circuits and Systems* (pp. 744–749). IEEE. https://doi.org/10.1109/MWSCAS.2005.1594208

Gupta, S., & Pingali, K. (1995). *Fast compiled logic simulation using linear BDDs* (Technical report). Cornell Theory Center. https://ecommons.cornell.edu/entities/publication/5febabad-b8ba-4db2-8716-41f3b12163ba

Huff, D., & Hanrahan, P. (2018). Using runtime circuit specialization to accelerate simulations of reconfigurable architectures. In *2018 International Conference on ReConFigurable Computing and FPGAs (ReConFig)* (pp. 1–6). IEEE. https://doi.org/10.1109/RECONFIG.2018.8641698

IEEE. (1993). *IEEE standard multivalue logic system for VHDL model interoperability (Std\_logic\_1164)* (IEEE Std 1164-1993). IEEE.

IEEE. (2024). *IEEE standard for SystemVerilog—Unified hardware design, specification, and verification language* (IEEE Std 1800-2023). IEEE. https://ieeexplore.ieee.org/document/10458102

Kabiri Chimeh, M., Hall, C. V., & O'Donnell, J. T. (2012). Optimisation and parallelism in synchronous digital circuit simulators. In *2012 IEEE 15th International Conference on Computational Science and Engineering* (pp. 94–101). IEEE. https://doi.org/10.1109/ICCSE.2012.23

Lague, S. (n.d.-a). *Digital Logic Sim* \[Computer software\]. itch.io. https://sebastian.itch.io/digital-logic-sim

Lague, S. (n.d.-b). *Digital-Logic-Sim* \[Source code\]. GitHub. Retrieved November 5, 2026, from https://github.com/SebLague/Digital-Logic-Sim

LLVM Project. (n.d.-a). *Building an ORC-based JIT: Starting out with KaleidoscopeJIT*. In *LLVM documentation*. https://llvm.org/docs/tutorial/BuildingAJIT1.html

LLVM Project. (n.d.-b). *LLVM benchmarking tips*. In *LLVM documentation*. https://llvm.org/docs/Benchmarking.html

Logisim-evolution. (n.d.). *Logisim-evolution* \[Source code\]. GitHub. https://github.com/logisim-evolution/logisim-evolution

Lööw, A. (2025). The simulation semantics of synthesisable Verilog. *Proceedings of the ACM on Programming Languages, 9*(OOPSLA1), Article 126. https://doi.org/10.1145/3720484

Maurer, P. M. (2003). Efficient event-driven simulation by exploiting the output observability of gate clusters. *IEEE Transactions on Computer-Aided Design of Integrated Circuits and Systems, 22*(11), 1471–1486. https://doi.org/10.1109/TCAD.2003.818305

McKeeman, W. M. (1998). Differential testing for software. *Digital Technical Journal, 10*(1), 100–107.

Miller, B. P., Fredriksen, L., & So, B. (1990). An empirical study of the reliability of UNIX utilities. *Communications of the ACM, 33*(12), 32–44. https://doi.org/10.1145/96267.96279

PlangoDev. (n.d.). *tiny-logic-doom* \[Source code\]. GitHub. https://github.com/PlangoDev/tiny-logic-doom

*Turing Complete*. (n.d.). https://turingcomplete.game/

Verilator. (n.d.-a). *Languages*. In *Verilator documentation*. https://github.com/verilator/verilator/blob/master/docs/guide/languages.rst

Verilator. (n.d.-b). *Overview*. In *Verilator documentation*. https://verilator.org/guide/latest/overview.html

Verilator. (n.d.-c). *Simulating*. In *Verilator documentation*. https://verilator.org/guide/latest/simulating.html

Verilator. (n.d.-d). *Verilator internals*. In *Verilator documentation*. https://verilator.org/guide/latest/internals.html

*Virtual Circuit Board*. (n.d.). Steam. https://store.steampowered.com/app/1885690/Virtual\_Circuit\_Board/

Wang, Z., & Maurer, P. M. (1990). LECSIM: A levelized event driven compiled logic simulation. In *Proceedings of the 27th ACM/IEEE Design Automation Conference* (pp. 491–496). IEEE Computer Society Press. https://doi.org/10.1145/123186.123349

whoismuzzy. (2026). *Tiny Logic* \[Early access video game\]. itch.io. https://whoismuzzy.itch.io/tinylogic

YosysHQ. (n.d.-a). *Memory library mapping*. In *Yosys documentation*. https://github.com/YosysHQ/yosys/blob/main/passes/memory/memlib.md

YosysHQ. (n.d.-b). *Model checking*. In *Yosys documentation*. https://yosyshq.readthedocs.io/projects/yosys/en/latest/using\_yosys/more\_scripting/model\_checking.html

## BAB III METODOLOGI DAN PERANCANGAN SISTEM

### 3.1 Metode Pengembangan

Tugas Akhir ini berbasis proyek, sehingga luarannya adalah perangkat lunak yang dapat dijalankan beserta bukti rekayasa yang mendukungnya. Karena itu bab ini memuat metode pengembangan perangkat lunak dan perancangan sistem, bukan metode penelitian eksperimental.

Metode yang dipakai adalah pengembangan inkremental yang digerakkan oleh risiko (*risk-driven*), dengan baseline arsitektur yang ditetapkan lebih dahulu. Pengembangan inkremental membangun sistem dalam potongan kecil yang masing-masing dapat dijalankan, sedangkan pengembangan iteratif memanfaatkan hal yang dipelajari dari potongan sebelumnya untuk memperbaiki potongan berikutnya. Kombinasi keduanya telah dipraktikkan sejak pertengahan 1950-an dan menghindari pendekatan satu kali jalan yang berurutan, digerakkan oleh dokumen, dan dibatasi oleh gerbang tahapan (Larman & Basili, 2003). Pengambilan keputusan yang digerakkan oleh risiko mengikuti gagasan model spiral, yaitu proses pengembangan dipandu oleh risiko proyek (Boehm, 1988). Pada proyek ini, risiko terbesar ada pada semantik simulasi, konektivitas, dan integrasi Godot dan Rust, sehingga hal-hal itulah yang dibuktikan paling awal melalui *vertical slice*.

Pengembangan dibagi menjadi milestone berbasis kemampuan (*capability-based*). Setiap milestone menghasilkan artefak yang dapat diperiksa, yaitu kode beserta *test*, demo, *benchmark*, atau dokumentasi keputusan, dan dinyatakan selesai hanya bila memenuhi *exit criteria* yang ditetapkan sebelumnya. Pendekatan ini sejalan dengan panduan NASA Software Engineering Handbook yang menekankan milestone dengan kriteria *deliverable* (NASA, n.d.-a) serta pemantauan risiko sepanjang siklus hidup proyek (NASA, n.d.-b). Panduan tersebut dipakai sebagai prinsip perencanaan, bukan sebagai proses birokratis yang disalin penuh. Karena biaya mengubah arsitektur sangat tinggi setelah implementasi berjalan, keputusan arsitektur utama ditetapkan lebih dahulu sebagai baseline (subbab 3.3.4). Baseline itu ditinjau pada setiap titik tinjau milestone dan hanya diubah melalui pengendalian perubahan.

Pada saat proposal ini disusun, kajian arsitektur yang menjadi dasar Bab II dan Bab III telah selesai, sedangkan implementasi produk belum dimulai. Milestone sengaja tidak diberi tanggal karena kalender akademik dan alokasi waktu mingguan belum ditetapkan. Milestone akan dipetakan ke jadwal Tugas Akhir setelah batas waktu diketahui.

#### 3.1.1 Prinsip Pengembangan

Tujuh prinsip berikut mengatur urutan dan prioritas pekerjaan.

**Tabel 3.1** Prinsip pengembangan dan penerapannya

| Prinsip | Penerapan pada proyek |
| --- | --- |
| *Vertical slice* lebih awal | Alur end-to-end dari dokumen rangkaian hingga output yang dapat diamati dibangun secepat mungkin |
| *Correctness* sebelum optimasi | Implementasi sederhana yang benar menjadi *baseline*, dan optimasi hanya diterima bila *differential test* serta *regression test* lulus dan *benchmark* menunjukkan manfaat |
| Core engine sebelum keluasan fitur | Model rangkaian, konektivitas, *compile*, *runtime*, *state* sekuensial, memori, dan *observability* didahulukan dari katalog komponen dan *polish* UI |
| Semantik tidak dikorbankan demi scope | Bila waktu terbatas, jenis komponen atau *workload* dikurangi, bukan aturan logic, *clock*, memori, atau *feedback* |
| Artefak di setiap milestone | Kode saja bukan *exit criteria*, harus ada *test*, demo, *benchmark*, atau dokumentasi keputusan |
| TBD memiliki pemilik dan *trigger* | TBD (to be determined) adalah keputusan yang belum ditetapkan. Keputusan yang belum perlu dibuat ditunda, sedangkan yang menghambat implementasi diselesaikan sebelum milestone terkait |
| Tanpa infrastruktur spekulatif | JIT, *plugin* ABI, *incremental compile*, dan *distributed simulation* tidak dibangun sebelum kebutuhan dan buktinya muncul |

Tabel 3.1 mengikat prinsip-prinsip tersebut pada pekerjaan yang konkret. Intinya terdiri atas tiga hal, yaitu kemampuan inti dan *correctness* didahulukan dari keluasan fitur, pengurangan scope tidak boleh mengorbankan semantik, dan setiap keputusan yang ditunda harus memiliki pemilik serta *trigger*. Ketiganya dipakai kembali pada pengendalian perubahan (3.1.3), daftar TBD (3.4.3), dan strategi *fallback* (3.8.1).

#### 3.1.2 Tahapan Proyek

1. **Kajian dan perancangan.** Kajian simulator pembanding, algoritma simulasi, arsitektur *runtime*, semantik, strategi validasi, dan risiko telah diselesaikan dan menjadi dasar bab ini.
2. **Penetapan baseline implementasi (M0).** Pilihan stack, struktur repository, dan alur *build* serta *test* ditetapkan, lalu dibuktikan melalui *prototype* integrasi.
3. **Implementasi inkremental (M1 sampai M6).** Model rangkaian, konektivitas, *compile*, *runtime*, *state* sekuensial, memori, editor, dan *simulation worker* dibangun bertahap dengan *correctness harness* sebagai gerbang.
4. **Evaluasi dan stabilisasi (M7 sampai M9).** *Benchmark*, demonstrasi CPU, *regression suite*, laporan kinerja, daftar keterbatasan, dan paket rilis disiapkan.

#### 3.1.3 Pemantauan dan Pengendalian Perubahan

Kemajuan diukur dengan *exit criteria* milestone, bukan dengan persentase fitur atau jumlah baris kode. Pemantauan minimum mencakup status milestone (*Not started*, *In progress*, *Blocked*, atau *Accepted*), pemetaan kebutuhan ke *test* untuk fitur inti, *risk register* yang memuat pemilik, *trigger*, dan status mitigasi, *decision log* untuk perubahan semantik, arsitektur, dan scope, *baseline benchmark* untuk setiap klaim peningkatan kinerja, serta daftar keterbatasan dan pekerjaan yang ditunda. Setiap perubahan scope diuji dengan lima pertanyaan, yaitu apakah perubahan diperlukan untuk P0 atau milestone aktif, apakah ia mengubah semantik atau batas modul, apa dampaknya pada *test*, *benchmark*, dan jadwal, fitur apa yang ditunda sebagai kompensasi, serta apakah keputusan rancangan terkait perlu diperbarui atau cukup dicatat sebagai *decision log*.

#### 3.1.4 Siklus Tiap Milestone dan Titik Tinjau

Setiap milestone diperlakukan sebagai satu iterasi dengan lima langkah berikut.

1. **Rencana.** Menetapkan *exit criteria*, *fixture* dan *test* yang akan menjadi bukti, serta risiko relevan dari *risk register*.
2. **Bangun.** Mengimplementasikan kemampuan milestone sebagai *vertical slice* yang dapat dijalankan, bukan komponen terpisah yang baru disatukan di akhir.
3. **Verifikasi.** Menjalankan *correctness suite*, dan *benchmark* bila milestone menyentuh jalur panas.
4. **Tinjau.** Mencatat keputusan dan temuan pada *decision log*, memperbarui *risk register*, lalu menilai apakah kebutuhan, scope, atau baseline arsitektur perlu diubah.
5. **Konsultasi.** Pada titik tinjau yang ditetapkan, hasil siklus dipaparkan kepada dosen pembimbing untuk memperoleh masukan.

Titik tinjau dengan dosen pembimbing diusulkan pada M0 (baseline implementasi), M3 (simulasi terkompilasi pertama), M6 (*runtime* interaktif), M7 (baseline *benchmark*), dan M9 (penyerahan akhir). Usulan ini akan disesuaikan dengan kesepakatan bimbingan yang kelak ditetapkan. Durasi tiap iterasi (*time-box*) ditetapkan setelah jadwal Tugas Akhir diketahui, sehingga pada tahap proposal milestone hanya diurutkan menurut ketergantungan.

Kebutuhan awal diturunkan dari kajian simulator pembanding, bukan dari survei pengguna. Untuk mengurangi risiko tersebut, kebutuhan divalidasi melalui evaluasi kebergunaan sederhana pada M6 dan M8, sebagaimana dijelaskan pada subbab 3.7.6 sebagai target P1.

### 3.2 Analisis Kebutuhan

Kebutuhan sistem diturunkan dari scope proyek (S1) yang ditetapkan pada kajian, lalu dikelompokkan menjadi tiga kelas prioritas. Kelas P0 adalah produk minimum yang harus selesai agar DLF benar-benar menjadi simulator. Kelas P1 memperkuat target teknis dan demonstrasi. Kelas P2 adalah pengembangan lanjutan yang tidak boleh mengancam penyelesaian inti.

#### 3.2.1 Pengguna dan Skenario Penggunaan

Pengguna sasaran adalah pembelajar arsitektur komputer dan pembangun rangkaian hobi yang ingin menyusun komputer dari gerbang logika secara visual. Skenario berikut disusun dari *workload suite* pada rancangan evaluasi, bukan dari survei pengguna.

1. Menyusun rangkaian kombinasional seperti gerbang, *multiplexer*, *adder*, dan ALU, lalu mengubah input dan mengamati output.
2. Membangun rangkaian sekuensial seperti register, *counter*, dan FSM (finite state machine, yaitu rangkaian yang berpindah antar-state menurut aturan tertentu) beserta memori, lalu menjalankannya siklus demi siklus dengan *run* atau *step*.
3. Membuat *chip* hierarkis dan memakainya ulang dengan *state* yang independen pada setiap *instance*.
4. Membangun CPU sederhana, memuat program, dan menjalankannya melalui jalur input dan tampilan.
5. Menelusuri kesalahan rangkaian melalui diagnostik yang menunjuk komponen, pin, atau *wire* penyebab.

&#91;embedded content: use case DLF · 7 interaksi pengguna\]

Gambar 3.1 merangkum tujuh *use case* utama. Menjalankan simulasi selalu menyertakan *compile* rangkaian, sedangkan kontrol simulasi mencakup *run*, *pause*, *step*, *reset*, dan *stop*.

Dari kelima skenario tersebut, skenario 1, 2, dan 5 dicakup oleh prioritas P0, sedangkan skenario 3 dan 4 merupakan target prioritas P1.

#### 3.2.2 Kebutuhan Fungsional Prioritas P0

**Tabel 3.2** Kebutuhan fungsional P0

| ID | Area | Kebutuhan | Kriteria penerimaan |
| --- | --- | --- | --- |
| KF-01 | Dokumen rangkaian | Dokumen dengan *stable ID*, *instance* komponen, pin, parameter, geometri *wire*, dan hierarki dasar | Dapat dibuat, disimpan, dimuat, dan divalidasi |
| KF-02 | Editor dasar | *Canvas*, *pan/zoom*, *select*, *place*, *move*, *delete*, *connect*, dan *inspector* properti | Rangkaian uji dapat disusun tanpa menyunting *file* secara manual |
| KF-03 | *Library* primitif | Gerbang dasar, *constant*, input dan output, *buffer*, *mux*, serta *splitter* dan *merger* | Arah pin, lebar, dan perilaku mengikuti kontrak komponen |
| KF-04 | Konektivitas | *Endpoint*, *junction*, segmen *wire*, pemetaan pin, lebar bus, dan resolusi net yang deterministik | Koneksi tidak valid menghasilkan *error compile* dan geometri tidak masuk *hot loop* |
| KF-05 | Pipeline *compile* | *Snapshot*, validasi struktural, elaborasi hierarki dasar, konektivitas, pemeriksaan semantik, *IR lowering*, dan alokasi *runtime* | *Compile* deterministik dan diagnostik memiliki *source mapping* yang memadai |
| KF-06 | *Runtime* | Representasi operasi ringkas, penyimpanan sinyal, *static schedule* untuk wilayah kombinasional tanpa siklus, dan *current/next state* | Tidak mengakses objek editor dan *correctness suite* dasar lulus |
| KF-07 | Sekuensial | Register ber-*clock* dan komponen sekuensial yang dibutuhkan *workload* inti | *Simultaneous state commit* dan semantik *clock* eksplisit |
| KF-08 | Memori | Sekurangnya satu bentuk RAM dan ROM dengan semantik terdokumentasi | Uji baca, tulis, dan inisialisasi lulus |
| KF-09 | Kontrol simulasi | *Compile-on-Simulate*, *run*, *pause*, *step*, *reset*, dan *stop* | Transisi *state* dan penanganan *error* deterministik |
| KF-10 | *Observability* | Nilai input dan output, *probe* terpilih, status *compile* dan *runtime*, serta diagnostik | Output dapat diverifikasi tanpa *debugger* internal |
| KF-11 | Validasi | *Reference evaluator* untuk subset semantik inti, *golden test*, dan *differential test* | Tidak ada regresi pada *suite* yang telah dikunci |
| KF-12 | *Benchmark* | Waktu *compile*, CPS inti, memori, dan *workload end-to-end* minimal | *Raw result* dan *manifest* tersimpan |

P0 tidak mensyaratkan seluruh komponen katalog, semua fitur editor, atau seluruh *workload benchmark* selesai. Yang disyaratkan adalah satu simulator yang konsisten untuk subset rangkaian yang dinyatakan didukung. Kontrak komponen dan antarmuka antarmodul dirinci pada subbab 3.10. Kontrol simulasi pada KF-09 dijalankan melalui *simulation worker* terpisah dengan perintah bertipe dan *snapshot*, sehingga UI tidak membaca *runtime state* secara langsung dan tetap responsif.

#### 3.2.3 Kebutuhan Prioritas P1 dan P2

Target P1 meliputi hal berikut.

- *Chip* hierarkis kustom yang dapat disimpan dan dipakai ulang dengan *state* yang independen pada setiap *instance*.
- Logika empat nilai dan resolusi beberapa *driver* pada cakupan komponen dan net yang ditetapkan.
- Dukungan *feedback* dan SCC sesuai semantik yang diputuskan. Struktur yang belum didukung menghasilkan diagnostik, bukan hasil palsu.
- Operasi bus dan multi-bit yang matang hingga lebar 64 bit.
- Penguatan *simulation worker* yang sudah menjadi bagian P0, berupa *command queue*, *batching* yang dapat disetel, dan penggabungan *snapshot*.
- *Probe* dan *trace* terpilih beserta ekspor hasil pengamatan.
- *Workload* CPU sederhana yang menjalankan *instruction loop* deterministik, serta demonstrasi CPU buatan pengguna. Subset RISC-V boleh menjadi target demonstrasi, tetapi DLF tidak wajib menyediakan *assembler* atau *compiler* sebagai fitur produk.
- *Benchmark* terhadap variasi *runtime*, *scheduling*, dan penyimpanan yang paling berpengaruh.

Kelas P2 mencakup JIT dan *native code generation*, *incremental compilation*, *public plugin API*, bahasa ekspresi parameter untuk *chip* generik, *multi-clock domain*, bus dengan lebar sangat besar, *waveform database* penuh, eksekusi paralel atau GPU, kompatibilitas RISC-V penuh, serta kolaborasi dan sinkronisasi *cloud*. Item P2 bukan daftar yang wajib diselesaikan setelah P1. Item tersebut hanya dipertimbangkan bila inti stabil dan ada kebutuhan nyata.

Selain butir di atas, evaluasi kebergunaan oleh pengguna juga menjadi target P1. Evaluasi ini memvalidasi bahwa pengguna sasaran dapat menyusun dan menjalankan rangkaian serta memahami diagnostik, dan rancangannya dijelaskan pada subbab 3.7.6.

#### 3.2.4 Kebutuhan Nonfungsional

**Tabel 3.3** Kebutuhan nonfungsional

| ID | Kebutuhan | Penjelasan |
| --- | --- | --- |
| KNF-01 | Determinisme | Hasil simulasi tidak bergantung pada urutan komponen, urutan penelusuran, atau ukuran *batch*, dan *compile* berulang pada dokumen yang sama menghasilkan hasil yang sama |
| KNF-02 | *Correctness* sebagai gerbang | Optimasi hanya diterima bila *correctness test* lulus dan *benchmark* menunjukkan manfaat |
| KNF-03 | *Diagnosability* | *Error* dapat ditelusuri ke komponen, pin, *wire*, atau jalur hierarki, dan rangkaian yang belum didukung ditolak, bukan disimulasikan secara diam-diam salah |
| KNF-04 | *Testability* dan modularitas | Model rangkaian, *compiler*, dan *runtime* dapat dibangun dan diuji tanpa menjalankan Godot |
| KNF-05 | *Responsiveness* | UI tetap responsif saat simulasi berjalan, dan hilangnya *frame* presentasi tidak menghilangkan siklus simulasi |
| KNF-06 | *Reproducibility* | *Benchmark* memakai *release build*, *manifest*, dan *raw result*, dan proyek dapat dibangun dari *clean checkout* |
| KNF-07 | Platform | Aplikasi desktop Windows 64-bit dengan target *x86\_64-pc-windows-msvc* |

Target numerik untuk CPS, latensi *compile*, batas memori, dan FPS aplikasi belum ditetapkan. Angka tersebut baru ditentukan setelah *baseline workload* dan hasil *prototype* tersedia, sehingga tidak ada klaim kinerja yang dibuat sebelum pengukuran.

#### 3.2.5 Library Komponen Awal

Library awal difokuskan pada komponen yang langsung mendukung pembangunan CPU, bukan pada kelengkapan katalog simulator lain. Komponen dikelompokkan menjadi lima lapisan pada Tabel 3.4, dan urutan implementasinya mengikuti kebutuhan *workload*.

**Tabel 3.4** Library komponen awal

| Lapisan | Komponen |
| --- | --- |
| Wiring dan inspeksi sinyal | Pin input dan output, *constant*, *probe*, *splitter*, *merger*, *bit extender* dan *truncator* yang eksplisit, *tunnel*, serta sumber *clock* dan *reset* |
| Gerbang logika (L0) | *Buffer*, NOT, AND, NAND, OR, NOR, XOR, XNOR, dan *tri-state buffer* |
| Kombinasional (L1) | *Multiplexer*, *demultiplexer*, *decoder*, *priority encoder*, *comparator*, *adder*, *subtractor*, *shifter*, dan *bit selector* |
| Sekuensial dan memori (L2) | *SR latch*, *D latch*, *D flip-flop*, register, *counter*, *shift register*, ROM, RAM satu *port*, dan *register file* |
| I/O dan sistem (L3) | *Toggle switch*, *button*, LED, *seven-segment display*, *LED matrix* atau *framebuffer display*, dan *keyboard* |

DLF mempertahankan dua jalur desain yang sama-sama valid. Pada jalur *gate-level construction*, pengguna membangun *adder*, register, dan ALU dari primitif dasar. Pada jalur *functional abstraction*, pengguna menempatkan komponen fungsional secara langsung untuk membangun sistem besar dengan lebih sedikit *wiring*. DLF perlu membedakan apakah sebuah blok adalah primitif *native*, *macro* yang diekspansi, atau *chip* kustom, supaya perilaku, kinerja, dan *debugging*-nya dapat dipahami. Pemodelan transistor dan perilaku analog, *propagation delay*, katalog TTL (transistor-transistor logic, keluarga chip logika klasik) lengkap, blok *floating-point*, IP vendor FPGA, serta impor HDL sembarang berada di luar library awal.

### 3.3 Perancangan Arsitektur Sistem

#### 3.3.1 Gambaran Umum

DLF dirancang sebagai simulator rangkaian visual dengan *compile-on-Simulate*, *execution plan* yang ringkas, dan *runtime state* yang terisolasi dari editor. Alur utamanya adalah sebagai berikut.

**Edit Mode → Immutable Circuit Snapshot → Compiler → Immutable Compiled Artifact → Runtime Instance → Simulation Worker → Immutable Output Snapshot → UI**

&#91;embedded content: arsitektur DLF · 8 komponen dari Edit Mode sampai UI\]

Gambar 3.2 memperlihatkan bahwa editor tidak pernah membaca atau mengubah *runtime state*. Hubungan antara UI dan *worker* hanya melalui *snapshot* yang *immutable* dan perintah bertipe.

Editor mengutamakan penyusunan rangkaian yang fleksibel dan hierarkis. Ketika pengguna menekan Simulate, *compiler* mengubah *snapshot* rangkaian menjadi artefak yang tidak dapat diubah (*immutable*), yang telah divalidasi, dielaborasi, dan dianalisis ketergantungannya. *Simulation worker* kemudian menjalankan artefak itu dengan *state* miliknya sendiri, sedangkan UI hanya berkomunikasi melalui perintah dan *snapshot*, tidak dengan membaca atau mengubah *state* internal *worker*.

Arsitektur ini dipilih karena *workload* utama DLF bukan hanya rangkaian kombinasional kecil, melainkan *datapath*, register, memori, CPU, dan aplikasi interaktif yang mengeksekusi banyak siklus. Biaya menelusuri objek editor, mencari koneksi, menelusuri hierarki secara rekursif, dan melakukan *dynamic dispatch* per komponen sebaiknya dibayar saat *compile*, bukan berulang kali pada *hot loop*. Perubahan struktur rangkaian tidak didukung selama simulasi berjalan. Pengguna menghentikan simulasi, kembali ke mode Edit, lalu melakukan *compile* ulang, sedangkan masukan *runtime* seperti *switch*, tombol, dan *keyboard* tetap dapat berubah tanpa *compile* ulang.

#### 3.3.2 Representasi Data

DLF memakai tiga representasi utama dan satu *snapshot* perantara, yang dirangkum pada Tabel 3.5. Pemisahan ini mencegah objek editor masuk ke jalur eksekusi.

**Tabel 3.5** Representasi data pada DLF

| Representasi | Isi | Mutabilitas |
| --- | --- | --- |
| *Authoring Document* | Hierarki, *instance* komponen, parameter, kontrak pin, geometri *wire*, *junction*, *tunnel*, dan metadata editor | *Mutable* pada mode Edit |
| *Circuit Snapshot* | Salinan konsisten dari dokumen beserta definisi dan aset yang dirujuk | *Immutable* |
| *Compiled Circuit Artifact* | Net kanonis, IR, *schedule* dan partisi, *runtime layout*, skema *state*, *source map*, serta versi semantik dan *compiler* | *Immutable* |
| *Runtime State* | Slot sinyal, *state* register dan *latch*, isi memori, *state clock* dan periferal, serta penghitung *runtime* | *Mutable*, dimiliki satu *simulation instance* |

Tabel 3.5 menegaskan satu aturan utama, yaitu hanya *Authoring Document* yang dapat diubah oleh pengguna, dan hanya *Runtime State* yang berubah saat simulasi berjalan. *Circuit Snapshot* dan *Compiled Circuit Artifact* bersifat *immutable*, sehingga *compiler* dan *runtime* tidak pernah membaca objek editor yang masih berubah. Entitas dokumen dan aturan ID dirinci pada subbab 3.10.1.

#### 3.3.3 Subsistem dan Batas Modul

Implementasi dapat dimulai sebagai satu aplikasi dan satu repository, tetapi batas logis antarsubsistem pada Tabel 3.6 harus dipertahankan. Batas ini menjaga agar editor, *compiler*, dan *runtime* tidak saling bergantung secara tidak terkendali.

**Tabel 3.6** Subsistem dan kontraknya

| Subsistem | Tanggung jawab | Larangan utama |
| --- | --- | --- |
| Project dan *Circuit Document* | Menyimpan hierarki, *instance*, parameter, pin, geometri *wire*, dan metadata dengan *stable ID* serta versi skema | Menyimpan *runtime state* aktif sebagai sumber kebenaran |
| Editor dan *Interaction* | *Canvas*, *selection*, *placement*, *wiring*, *inspector*, *undo/redo* | Menghitung hasil simulasi sendiri |
| *Snapshot/Compile Controller* | Membekukan *snapshot* dan mengelola status *compile* | Membiarkan *compiler* membaca dokumen yang sedang berubah |
| *Compiler Frontend* | Validasi, elaborasi hierarki, dan konektivitas kanonis beserta diagnostik | Bergantung pada objek *renderer* atau *canvas* |
| *Semantic/Graph Analysis* | Menganalisis *driver*, batas sekuensial, ketergantungan kombinasional, dan SCC | Mengasumsikan seluruh graf adalah DAG (graf berarah tanpa siklus) |
| *IR Lowering/Optimizer* | Menurunkan model tervalidasi menjadi IR dan *execution plan* | Menghapus perilaku yang dapat diamati atau melanggar semantik |
| *Runtime Layout/Artifact* | Membentuk artefak *immutable* dan rencana alokasi *runtime* | Menyimpan *state* simulasi yang berubah di dalam artefak |
| *Runtime Engine* | Menjalankan transisi *state* yang deterministik | Mengakses objek editor atau melakukan *rendering* UI |
| *Simulation Worker* | Siklus hidup eksekusi, perintah bertipe, dan *output snapshot* | Membagikan *state* internal yang dapat berubah ke UI |
| *Observability/Presentation* | Nilai UI, *trace*, dan penghitung kinerja dari *snapshot* | Memaksa *full trace* pada setiap siklus di jalur cepat |
| *Validation/Benchmark Harness* | *Fixture*, *oracle*, dan *manifest* untuk laporan kebenaran dan hasil mentah kinerja | Mencampur metrik *compile*, *runtime*, *rendering*, dan aplikasi |

Kolom larangan pada Tabel 3.6 menunjukkan arah ketergantungan yang diizinkan. Editor dan UI tidak menghitung hasil simulasi, *compiler* tidak membaca objek *renderer*, dan *runtime* tidak mengakses objek editor, sehingga setiap subsistem dapat diuji tanpa subsistem di atasnya. Batas inilah yang memungkinkan *core* dibangun dan diuji tanpa Godot.

#### 3.3.4 Keputusan Arsitektur yang Dikunci

Keputusan pada Tabel 3.7 dikunci pada tingkat arsitektur dan menjadi batasan implementasi. Perubahan hanya dapat dilakukan melalui pengendalian perubahan bila bukti implementasi menunjukkan konflik pada kebenaran, kinerja, atau kelayakan.

**Tabel 3.7** Keputusan arsitektur yang dikunci

| ID | Keputusan |
| --- | --- |
| D01 | Mode Edit dan Simulate dipisahkan, dan Simulate memicu *compile* eksplisit |
| D02 | *Authoring Document*, *Compiled Artifact* yang *immutable*, dan *Runtime State* yang *mutable* adalah representasi yang berbeda |
| D03 | *Compiler* hanya membaca *snapshot* yang *immutable* |
| D04 | *Runtime State* dimiliki secara eksklusif oleh *simulation worker* |
| D05 | Komunikasi *worker* memakai perintah bertipe serta *snapshot* atau kanal output yang *immutable* |
| D06 | Geometri *authoring* dipisahkan dari konektivitas elektris |
| D07 | Definisi komponen, *instance*, dan implementasi terkompilasi adalah entitas terpisah |
| D08 | Hierarki *chip* dipertahankan di *authoring*, sedangkan *runtime* boleh meng-*flatten* atau mengkhususkannya |
| D09 | *Runtime baseline* memakai IR ringkas dan penyimpanan kontigu tanpa *dispatch* berorientasi objek per komponen pada *hot loop* |
| D10 | Wilayah kombinasional tanpa siklus memakai jalur cepat eksekusi statis berurutan level |
| D11 | *Feedback* dan SCC mengikuti kebijakan semantik eksplisit, tanpa urutan acak atau DAG palsu |
| D12 | Logika sekuensial memakai *current state* dan *next state* dengan *simultaneous commit* |
| D13 | Memori memiliki representasi penyimpanan *native* dan semantik eksplisit |
| D14 | *Full compile* dari *snapshot* adalah *baseline* awal |
| D15 | JIT dan *native code generation* bukan *baseline* awal |
| D16 | *Correctness* adalah gerbang sebelum kinerja |
| D17 | Rangkaian yang tidak didukung ditolak dengan diagnostik, bukan menghasilkan simulasi yang diam-diam salah |
| D18 | Kinerja dilaporkan terpisah untuk *compile*, *core runtime*, memori, UI, dan aplikasi *end-to-end* |

Kedelapan belas keputusan pada Tabel 3.7 dapat dibaca dalam tiga kelompok. D01 sampai D08 mengatur pemisahan dan kepemilikan data, D09 sampai D15 mengatur pilihan teknik *runtime* dan *compile*, dan D16 sampai D18 mengatur *correctness* serta pelaporan kinerja. D14 dan D15 sengaja konservatif, yaitu *full compile* menjadi *baseline* dan JIT tidak dipakai, agar kompleksitas baru ditambahkan setelah ada bukti kebutuhan.

#### 3.3.5 Stack Teknologi

Stack dipilih berdasarkan kesesuaian dengan kebutuhan DLF, biaya integrasi dan pemeliharaan untuk satu pengembang utama, produktivitas berdasarkan kemampuan pengembang saat ini, serta risiko yang dapat diuji melalui *prototype*. DLF diperlakukan sebagai simulator digital yang memakai Godot sebagai lapisan presentasi (*presentation layer*), bukan sebagai proyek Godot yang menjadikan *engine* pemilik logika domain. Semantik rangkaian, *compiler*, dan *runtime* harus tetap dapat diuji tanpa menjalankan Godot.

**Tabel 3.8** Baseline stack teknologi

| Lapisan | Teknologi | Tanggung jawab |
| --- | --- | --- |
| Platform | Windows 64-bit desktop, target Rust *x86\_64-pc-windows-msvc* | Membatasi cakupan kompatibilitas dan pengujian awal |
| UI framework | Godot 4.7.2 *stable* (*standard build*, non-.NET) | Jendela, *scene*, input, *rendering*, panel, menu, *viewport*, dan ekspor proyek |
| UI glue | GDScript dengan *static typing* | Menghubungkan *scene*, *signal*, interaksi input, dan *facade* Rust, tanpa memuat logika domain |
| Bahasa inti | Rust *stable*, edisi 2024 | Model rangkaian, konektivitas, validasi, *compiler*, IR, *runtime*, memori, *worker*, dan model perintah dan *snapshot* |
| *Binding* Godot | Crate `godot` dari godot-rust 0.5.5 dengan *API level* `api-4-7` | Implementasi GDExtension dan API Rust yang dipanggil dari GDScript |
| *Build* dan dependensi | *Cargo workspace* | *Build*, *test*, *lint*, dan *benchmark* Rust |
| Distribusi | Installer Windows (`.exe`), dengan Inno Setup sebagai kandidat | Pengemasan dikerjakan pada milestone terakhir dan bukan penghambat inti |

GDScript adalah bahasa skrip bawaan Godot, sedangkan GDExtension adalah mekanisme Godot untuk memuat pustaka *native*, dalam hal ini Rust, ke dalam proses yang sama. Dengan mekanisme ini tidak ada proses *backend* atau IPC (inter-process communication, komunikasi antarproses) terpisah. Pasangan versi dipilih sebagai Godot 4.7.2 *stable* dan godot-rust 0.5.5, bukan Godot 4.8 yang saat keputusan diambil masih berstatus *beta*. Versi Rust, *crate*, dan *engine* dikunci melalui `rust-toolchain.toml` dan `Cargo.lock` agar *build* dapat direproduksi.

Stack ini bersifat *provisional implementation baseline*, bukan bukti bahwa integrasi Godot dan Rust, kinerja *canvas*, atau kinerja simulator telah tervalidasi. Godot adalah *game engine*, bukan *framework* desktop konvensional, sehingga fitur seperti dialog *file*, menu *native*, dan integrasi jendela mungkin memerlukan penyesuaian. godot-rust adalah *binding* komunitas yang dapat mengalami perubahan API antarversi (godot-rust, n.d.), dan dokumentasi Godot menyatakan bahwa *scene tree* aktif tidak aman untuk banyak *thread* (Godot Engine, n.d.). Risiko ini diterima dengan membatasi integrasi pada satu *crate extension*, mengunci versi, dan membuktikan kompatibilitasnya melalui *prototype* pada M0 sebelum arsitektur UI bergantung luas padanya.

#### 3.3.6 Pembagian Kepemilikan antara Rust dan Godot

**Tabel 3.9** Pembagian kepemilikan data dan tanggung jawab

| Data atau tanggung jawab | Pemilik |
| --- | --- |
| Dokumen rangkaian logis serta *stable ID* komponen dan pin | Rust *core* |
| Konektivitas, resolusi net, *compiler*, IR, dan artefak hasil *compile* | Rust *core*, tanpa bergantung pada *scene tree*, *renderer*, atau objek Godot |
| *Runtime state* dan memori | *Simulation worker* Rust |
| Geometri visual, posisi dan ukuran komponen, serta geometri perutean *wire* | Lapisan UI Godot |
| *Selection*, *hover*, *viewport*, *zoom/pan*, dan *state* presentasi *inspector* | Lapisan UI Godot |
| *Spatial index* untuk *hit-testing* visual | Lapisan UI Godot, dengan *uniform-grid spatial hash* sebagai *baseline* awal |
| Presentasi output simulasi | Lapisan UI Godot yang membaca *snapshot* |

Identitas objek yang menghubungkan kedua sisi adalah *stable ID* dari model rangkaian. Godot boleh menyimpan catatan visual berdasarkan ID tersebut, tetapi tidak boleh menciptakan identitas elektris kedua yang bertentangan dengan Rust.

#### 3.3.7 Struktur Repository

Proyek memakai satu repository Git dan satu *Cargo workspace*. Subsistem tidak dipecah menjadi banyak *crate* sebelum batas dependensi dan kebutuhan penggunaan ulang terbukti.

- `crates/dlf-core` memuat model rangkaian, konektivitas, *compiler*, IR, dan *runtime*. *Crate* ini tidak boleh bergantung pada *crate* `godot`, sehingga dapat dibangun dan diuji sebagai pustaka Rust biasa.
- `crates/dlf-godot` adalah adaptor tipis GDExtension yang bergantung pada `dlf-core`.
- `app/godot` memuat proyek Godot, yaitu *scene*, *script*, sumber daya, dan *test* UI.
- `tests/fixtures`, `docs/` (arsitektur, keputusan, dan semantik), serta `scripts/` berisi skrip PowerShell untuk *build*, *test*, dan pengemasan.

### 3.4 Perancangan Pipeline Compile, Runtime, dan Semantik

Subbab ini menerjemahkan kajian pada Bab II menjadi rancangan yang akan diimplementasikan. Teori yang telah dibahas tidak diulang, dan fokusnya adalah keputusan rancangan DLF beserta kontraknya.

#### 3.4.1 Tahapan Compile

Alur *compile* baseline berjalan dari dokumen rangkaian hingga artefak yang siap di-*instantiate*.

**Circuit Document → Snapshot → Structural Validation → Hierarchy Elaboration → Connectivity Construction → Semantic Analysis → Combinational Dependency Analysis → IR Lowering → Runtime Layout Allocation → Compiled Artifact**

&#91;embedded content: pipeline compile · 9 tahap dari snapshot sampai artefak\]

Gambar 3.3 merangkum sembilan tahap *compile*. Tahap pertama membuat Circuit Snapshot, yang pada Gambar 3.2 digambar terpisah karena menjadi masukan compiler. Tahap kesembilan menghasilkan *Compiled Artifact* yang menjadi masukan *runtime*, dan kegagalan pada tahap mana pun menghentikan proses tanpa menghasilkan artefak.

**Tabel 3.10** Tahapan *compile* dan kontraknya

| Tahap | Kontrak |
| --- | --- |
| *Snapshot* | Mengambil versi dokumen yang konsisten beserta *revision* atau *hash*, definisi komponen, parameter, dan aset yang dirujuk. Perubahan editor setelahnya tidak memengaruhi *compile* yang sedang berjalan |
| *Structural validation* | Memeriksa definisi atau skema yang hilang, pin dan parameter tidak valid, ekspresi lebar yang gagal, instantiation *chip* rekursif, dan kesalahan struktur lain, dengan diagnostik yang menyimpan lokasi sumber |
| *Hierarchy elaboration* | Menyelesaikan *chip* kustom dan pemetaan *port*. *Runtime* boleh meng-*flatten* atau mengkhususkan hierarki, tetapi *source map* wajib mempertahankan jalur hierarki dan setiap *instance* yang menyimpan *state* tetap memiliki *state* independen |
| *Connectivity construction* | Membangun net kanonis dan pemetaan bit dari pin, *wire*, *junction*, *tunnel*, *splitter*, dan *merger* |
| *Semantic analysis* | Menerapkan profil semantik DLF untuk nilai 0/1/X/Z, resolusi *driver*, batas sekuensial, *clock* dan *reset*, *feedback*, dan memori |
| *Combinational dependency analysis* | Memisahkan ketergantungan kombinasional dari batas *state* sekuensial, mengidentifikasi SCC dan wilayah tanpa siklus, dan tidak mengasumsikan graf selalu DAG |
| *IR lowering* | Menurunkan komponen *built-in* menjadi *native kernel*, *operation descriptor*, atau ekspansi *macro*, dengan *constant folding*, *dead logic elimination*, dan optimasi lain hanya bila aman terhadap semantik dan kebutuhan *observability* |
| *Runtime layout allocation* | Mengalokasikan slot sinyal dan *state*, wilayah memori, *binding* input dan output, partisi *scheduler*, metadata *probe*, dan *source map* secara kontigu |
| *Artifact finalization* | Menyimpan versi *compiler* dan semantik, *fingerprint* rangkaian, *execution plan*, *layout*, diagnostik, dan *source map*, dan artefak hanya dipakai bila *fingerprint* serta versinya kompatibel |

Kegagalan *compile* tidak menghasilkan artefak yang dapat dijalankan. *Error* dipetakan kembali ke komponen, pin, *wire*, atau jalur hierarki yang relevan, dan *warning* yang tidak menghalangi eksekusi dibedakan dari *error* fatal.

#### 3.4.2 Aturan Konektivitas Visual

Geometri gambar rangkaian dipisahkan dari konektivitas elektris. Aturan *baseline* berikut membuat hasil konektivitas deterministik dan terdokumentasi.

1. *Endpoint wire* yang menempel pada pin terminal terhubung ke pin tersebut.
2. Dua *endpoint wire* yang bertemu pada koordinat yang sama terhubung.
3. *Endpoint wire* yang menyentuh bagian tengah *wire* lain membentuk *junction* dan memecah segmen yang dilintasi secara topologis.
4. Persilangan dua *wire* di bagian tengah tanpa *endpoint* atau *junction* eksplisit tidak terhubung.
5. *Junction* dengan tiga cabang atau lebih menggabungkan seluruh cabang dalam satu konektivitas.
6. *Wire* yang hanya berdekatan secara visual tetapi tidak berbagi *endpoint* atau *junction* tidak terhubung.
7. *Tunnel* menghubungkan *tunnel* bernama identik dalam satu *scope* rangkaian. *Tunnel* dari hierarki yang berbeda tidak otomatis terhubung kecuali melalui *port* hierarki yang eksplisit.
8. Koneksi yang tidak valid tetap dapat disimpan pada mode Edit untuk memudahkan perbaikan, tetapi *compiler* menghasilkan diagnostik dan menolak pembuatan artefak.

Bila aturan ini diterapkan pada saat *compile*, geometri dinormalisasi, segmen dipecah pada titik sambungan, node dibuat untuk setiap pin, bit, dan *endpoint*, lalu node yang terhubung digabungkan. *Splitter*, *merger*, dan komponen konversi lebar menghasilkan pemetaan bit yang eksplisit, lebar divalidasi, dan setiap kelompok konektivitas memperoleh *Net ID* kanonis. Setelah itu *driver* dan *load* diklasifikasikan dan aturan elektris diperiksa. *Union-Find* atau *Disjoint Set Union* (DSU), yaitu struktur data untuk menggabungkan elemen ke dalam kelompok yang saling terhubung, menjadi kandidat algoritma sementara untuk penggabungan node. DSU hanya dipakai selama *compile*, bukan sebagai representasi *runtime*, dan pilihan finalnya ditentukan melalui pengukuran memori dan waktu *compile*.

#### 3.4.3 Profil Semantik Baseline

Semantik simulasi ditetapkan sebagai kontrak yang harus dipatuhi oleh *compiler* dan *runtime*. Profil *baseline* pada Tabel 3.11 menjadi dasar pembuatan *test fixture* dan *reference evaluator*.

**Tabel 3.11** Profil semantik baseline

| Aspek | Keputusan baseline |
| --- | --- |
| Nilai sinyal | Empat nilai per bit, yaitu 0, 1, X, dan Z. Nilai berkekuatan sembilan tingkat seperti IEEE 1164 tidak diperlukan pada *runtime* awal |
| Makna X dan Z | X berarti tidak diketahui atau konflik, dan Z berarti *high impedance*. Z yang masuk ke logika biasa diperlakukan sebagai X |
| Beberapa *driver* | Resolusi eksplisit pada net. *Driver* 0 dan 1 yang bertentangan menghasilkan X, dan *driver* Z tidak bersaing dengan *driver* aktif. *Driver* biasa yang ganda menghasilkan diagnostik *compile*, sedangkan *driver* ganda yang disengaja memerlukan semantik *tri-state* atau net yang diresolusi |
| Wilayah kombinasional tanpa siklus | Eksekusi statis berurutan level (*levelized*) |
| *Feedback* | Dipisahkan menjadi wilayah SCC dan diselesaikan secara iteratif. *Feedback* yang stabil diizinkan, dan osilasi atau ketidakkonvergenan didiagnosis |
| Elemen sekuensial | Membaca *state* saat ini dan menyimpan *state* berikutnya secara serentak pada batas *clock*. *Rising edge* atau *falling edge* adalah properti komponen yang eksplisit |
| *State* awal | *State* sekuensial yang belum diinisialisasi bernilai X pada mode ketat. Nilai awal eksplisit dan mode inisialisasi nol yang deterministik dapat disediakan |
| *Reset* | *Reset* sinkron dan asinkron adalah semantik yang berbeda, dan *reset* asinkron tidak ditunda hingga *clock* berikutnya |
| Memori | Perilaku *read-during-write* eksplisit. Usulan bawaan untuk RAM satu *port* sinkron adalah *read-first* (mengembalikan isi lama), dan mode lain dinyatakan sebagai properti komponen |
| Lebar bus | Ketidakcocokan lebar, *truncation*, dan *extension* implisit tidak diterima secara diam-diam |
| Di luar cakupan | *Timing* analog, *metastability*, *setup/hold*, dan penundaan fisik |

Beberapa hal masih TBD dan harus ditutup sebelum spesifikasi semantik final dibuat. Komponen yang membutuhkan keputusan tersebut belum boleh menyatakan dukungannya.

Daftar berikut mencatat setiap TBD beserta batas penutupannya. Seluruhnya dimiliki penulis sebagai pengembang tunggal, penutupnya dirangkum per milestone pada subbab 3.10.13, dan keputusannya dicatat pada *decision log*.

- **TBD-1, tabel kebenaran primitif.** Mencakup perilaku *mux* saat *select* bernilai X atau Z. Ditutup sebelum M3, yaitu sebelum primitif kombinasional diimplementasikan.
- **TBD-2, pengodean empat nilai.** Pilihan antara *mask* dan pengodean dua bit terkemas ditentukan melalui eksperimen pada M3, dan paling lambat ditutup sebelum M4.
- **TBD-3, algoritma konvergensi SCC dan batas iterasi.** Ditutup sebelum *solver* SCC dikerjakan, dengan *trigger* berupa tersedianya *workload* W07.
- **TBD-4, kebijakan *state* awal.** Ditutup sebelum M4.
- **TBD-5, prioritas *reset* dan *enable* untuk setiap primitif sekuensial.** Ditutup sebelum M4.
- **TBD-6, mode baca RAM bawaan untuk setiap tipe memori.** Ditutup sebelum M4.
- **TBD-7, primitif *pull-up* dan *pull-down*.** Ditunda hingga ada *workload* yang membutuhkan *tri-state* atau net yang diresolusi.
- **TBD-8, kebijakan *multi-clock*.** Ditunda sebagai P2 hingga ada kebutuhan *workload* yang nyata.

#### 3.4.4 Siklus Eksekusi dan Simulation Worker

Urutan konseptual satu siklus simulasi menjadi kontrak tingkat arsitektur, dan detail semantiknya tunduk pada profil pada Tabel 3.11.

1. Mengambil dan menerapkan masukan *runtime* atau perintah *host* yang berlaku pada batas siklus.
2. Mengevaluasi logika kombinasional berdasarkan *state* saat ini dan masukan yang berlaku, termasuk penyelesaian *feedback* sesuai kebijakan semantik.
3. Menghitung *next state* seluruh komponen sekuensial dari nilai yang diamati pada siklus tersebut.
4. Menyimpan *next state* secara serentak pada batas *clock* yang relevan, bukan memperbarui register satu per satu menurut urutan penelusuran.
5. Memperbarui *state* dan output periferal yang dijadwalkan pada batas tersebut.
6. Memperbarui penghitung dan mengumpulkan data *probe* atau output yang berubah.
7. Melanjutkan ke siklus berikutnya atau menerbitkan *snapshot* sesuai kebijakan *worker*.

*Worker* menjalankan beberapa siklus dalam satu *batch* sebelum memeriksa kebutuhan presentasi atau perintah yang tidak harus diterapkan per siklus. Ukuran *batch* tidak boleh mengubah hasil semantik, dan perintah yang memengaruhi nilai input diterapkan pada batas yang deterministik. Kontrak komunikasi antara UI dan *worker* ditetapkan sebagai berikut.

1. GDScript memanggil *facade* GDExtension pada *main thread*.
2. *Facade* memvalidasi argumen dasar lalu mengirim perintah bertipe ke *worker* Rust melalui *channel*. Pemanggilan API tidak menunggu simulasi selesai.
3. *Simulation worker* hanya memegang data milik Rust dan *Runtime State*. *Worker* tidak menyimpan atau memanggil objek Godot seperti `Gd<T>`, *Node*, *SceneTree*, atau *RenderingServer*.
4. *Worker* menerbitkan *output snapshot* yang *immutable* atau digabung (*coalesced*), dan *main thread* mengambil *snapshot* terbaru pada waktu yang aman untuk memperbarui UI.
5. *Snapshot* presentasi boleh menggabungkan beberapa perubahan dan melewati *snapshot* lama bila UI tertinggal. Penangkapan *trace* atau *probe* yang harus lengkap memakai jalur terpisah dengan *buffer* dan kebijakan *overflow* yang eksplisit.
6. Perintah *pause*, *step*, *reset*, *stop*, dan input eksternal memiliki urutan yang deterministik, dan antrean perintah tidak mengubah *runtime state* dari *main thread* secara langsung.

API antara Godot dan Rust dirancang bergaya *typed high-level* dengan operasi *coarse-grained* atau *batch* untuk perubahan yang secara semantik merupakan satu transaksi, misalnya *paste*, pemindahan banyak objek sekaligus, atau pembuatan sekumpulan komponen dan koneksi. API yang terlalu halus, yang memanggil Rust sekali per pin atau per elemen, dihindari. Untuk data besar dipakai *typed array* atau *packed array*, bukan *Dictionary* atau *Variant* per elemen pada jalur panas.

&#91;embedded content: urutan pesan Simulate hingga kontrol · 5 pelaku\]

Gambar 3.4 memperlihatkan urutan pesan saat pengguna menekan Simulate. *Compile* berjalan lebih dahulu. Bila berhasil, *worker* dimulai dan menerbitkan *Output Snapshot* ke UI, sedangkan perintah pengguna berikutnya dikirim sebagai perintah bertipe melalui *channel*.

#### 3.4.5 Penanganan Kegagalan

Kegagalan *compile* mengembalikan diagnostik tanpa memulai *worker*. Kegagalan saat *runtime* menghentikan atau menangguhkan *worker* sesuai klasifikasi kesalahannya, menerbitkan diagnostik dan *snapshot* terakhir yang konsisten, lalu memungkinkan *reset* atau *restart*. Perintah *stop* menghentikan *worker* dan melepaskan *Runtime State* tanpa merusak *Authoring Document* maupun *Compiled Artifact*. Prinsip yang tidak boleh dilanggar adalah bahwa rangkaian dengan semantik yang belum didukung tidak ditampilkan seolah-olah valid. *Compiler* menolak rangkaian tersebut atau memberi diagnostik yang jelas.

### 3.5 Perancangan UI dan Interaksi

#### 3.5.1 Pendekatan Editor dan Rendering

Editor memakai pendekatan hibrida pada Godot. *Canvas* rangkaian dibuat khusus, sedangkan *toolbar*, menu, *inspector*, panel hierarki, dialog, dan panel pendukung memakai *Control node* serta komponen UI bawaan Godot. Rangkaian tidak digambar sebagai satu *node* untuk setiap gerbang, pin, atau *wire*. Sebuah *centralized custom renderer* menggambar data visual melalui API gambar Godot pada satu atau beberapa lapisan, misalnya untuk *wire*, simbol, *selection*, dan *overlay*, sehingga elemen rangkaian tidak berubah menjadi ribuan *node* individual.

Penggambaran ulang memakai *invalidation*. Perubahan visual menandai lapisan atau wilayah yang perlu digambar ulang, bukan memaksa seluruh UI bekerja tanpa perubahan. *Hit-testing* dan *selection* dipisahkan dari proses menggambar, dan indeks spasial di sisi Godot memakai *uniform grid* atau *spatial hash* agar kueri pointer tidak memindai seluruh elemen. Pilihan ini adalah *baseline* sederhana yang harus diuji pada rangkaian besar, dan belum ada klaim bahwa ia merupakan struktur spasial tercepat.

#### 3.5.2 Area UI dan Fungsinya

**Tabel 3.12** Area UI dan fungsinya

| Area | Fungsi | Implementasi |
| --- | --- | --- |
| *Canvas* rangkaian | Menempatkan, memindahkan, menghubungkan, dan menghapus komponen, serta *pan/zoom* dan *selection* | *Renderer* khusus dengan *spatial index* |
| Palet komponen | Memilih komponen dari *library* | *Control node* Godot |
| *Inspector* | Mengubah parameter dan properti komponen terpilih, misalnya lebar bus, jenis *edge*, dan mode *reset* | *Control node* Godot |
| Panel hierarki | Menelusuri *chip* dan *instance* kustom | *Control node* Godot |
| Kontrol simulasi | *Simulate*, *run*, *pause*, *step*, *reset*, dan *stop* | *Control node* yang memanggil *facade* Rust |
| Panel input dan output | Mengubah *switch*, tombol, dan *keyboard* sebagai masukan *runtime*, serta menampilkan LED, *display*, dan nilai *probe* | *Control node* dan *renderer* yang membaca *snapshot* |
| Panel diagnostik | Menampilkan *error* dan *warning* *compile* serta diagnostik *runtime* | *Control node* dengan tautan ke objek visual |

Pembagian pada Tabel 3.12 memisahkan bagian yang digambar sendiri dari bagian yang memakai komponen bawaan Godot. Hanya *canvas* yang memakai *renderer* khusus, karena hanya area itu yang berisi banyak elemen dan membutuhkan *hit-testing* cepat. Panel lain cukup memakai *Control node* sehingga tidak perlu dibangun dari awal. Tata letaknya digambarkan pada gambar berikut.

&#91;embedded content: wireframe UI editor · 5 area\]

Gambar 3.5 menunjukkan tata letak UI yang diusulkan. Area berwarna biru adalah *canvas* yang digambar oleh *renderer* khusus, sedangkan panel lainnya memakai *Control node* Godot.

#### 3.5.3 Mode, Operasi Penyuntingan, dan Pemetaan Diagnostik

UI memiliki dua mode, yaitu Edit dan Simulate. Pada mode Edit, pengguna mengubah *Authoring Document*, dan semua perubahan struktur terjadi di sini. Penekanan Simulate memicu *compile* eksplisit, lalu pada mode Simulate struktur rangkaian tidak dapat diubah, sedangkan masukan *runtime* tetap dapat dioperasikan. Operasi yang secara semantik merupakan satu transaksi, seperti *paste*, pemindahan banyak objek sekaligus, dan pembuatan sekumpulan komponen dan koneksi, dikirim ke Rust sebagai satu operasi *batch*, dan editor menyediakan *undo/redo*.

Untuk mencegah pengguna mengira bahwa setiap persilangan garis berarti koneksi, *junction* aktif ditampilkan dengan titik, dan persilangan tanpa koneksi diberi tampilan visual yang tidak ambigu. Koneksi tidak valid tetap dapat disimpan pada mode Edit, tetapi *compiler* menolaknya dengan diagnostik. Diagnostik tersebut dipetakan melalui *source map* ke komponen, pin, *wire*, atau jalur hierarki, dan panel diagnostik memungkinkan pengguna menyorot objek penyebabnya pada *canvas*.

Presentasi nilai pada UI hanya memakai *probe* terpilih dan *snapshot* terbaru dari *worker*. Perekaman seluruh sinyal pada setiap siklus tidak dilakukan pada jalur cepat. *Trace* yang harus lengkap memakai jalur terpisah, sehingga *frame rate* UI tidak membatasi kecepatan simulasi.

#### 3.5.4 Prototype Canvas pada M0

Sebelum editor lengkap dibangun, sebuah *prototype canvas* terpusat dibuat untuk membuktikan bahwa rangkaian besar dapat digambar, di-*pan* dan di-*zoom*, diuji *hit-testing*-nya, dan dipilih. Hasilnya dicatat sebagai *baseline*, dan tidak ada angka kinerja yang diklaim sebelum pengukuran dilakukan. Bila *prototype* ini atau integrasi Godot dan Rust gagal memenuhi kriteria M0, keputusan stack ditinjau ulang sebelum M1 dan M5 bergantung terlalu jauh padanya.

### 3.6 Rencana Implementasi dan Milestone

#### 3.6.1 Milestone dan Kriteria Keluar

Milestone pada Tabel 3.13 bersifat berbasis kemampuan, bukan *sprint* atau kalender. Setiap milestone harus selesai dengan artefak yang dapat diperiksa. Kode W01 sampai W12 dan B0 sampai B4 pada tabel ini dijelaskan pada subbab 3.7, sedangkan kode KF dan KNF dijelaskan pada subbab 3.2.

**Tabel 3.13** Milestone implementasi

| Milestone | Deliverable utama | *Exit criteria* |
| --- | --- | --- |
| M0 Implementation baseline | *Decision log* semantik, ID kebutuhan, kerangka *repository*, *build*, dan *test*, serta *scope baseline* | Tidak ada ambiguitas yang menghalangi *baseline* primitif, kombinasional, dan sekuensial, dan lingkungan dapat *build* serta menjalankan *test* |
| M1 Headless circuit model | Skema dokumen rangkaian, *stable ID*, *save/load*, dan *structural validator* | *Fixture* rangkaian dapat di-*round-trip* dan struktur tidak valid menghasilkan diagnostik |
| M2 Connectivity and semantic frontend | Model pin dan net, resolusi *wire* dan *junction*, validasi lebar, serta analisis ketergantungan dan SCC awal, ditambah elaborasi hierarki dan nilai X dan Z dasar sebagai target P1 | *Fixture* konektivitas menghasilkan pemetaan net yang diharapkan dan referensi sumber tersedia |
| M3 First compiled simulation | *Snapshot* → *compile* → IR ringkas → *runtime* → output yang dapat diamati | Rangkaian kombinasional non-trivial berjalan dan *reference test* serta *differential test* lulus |
| M4 Stateful simulation | *Clock*, register, *current/next state*, dan RAM/ROM awal | Uji *counter*, *shift register*, dan memori lulus pada urutan siklus yang deterministik |
| M5 Editor vertical slice | Penyuntingan *canvas*, penempatan primitif, *wiring*, *inspector*, dan *save/load* | Rangkaian dapat dibuat melalui UI dan menghasilkan output yang sama dengan *fixture* tanpa kepala (*headless*) |
| M6 Interactive runtime | *Worker*, *command queue*, *run/pause/step/reset/stop*, *probe*, dan *snapshot* | UI tetap responsif, serta semantik perintah dan kepemilikan *state* tervalidasi |
| M7 Benchmark baseline | Subset W01 sampai W09 (W07 dan W08 hanya bila fitur P1 terkait selesai), *baseline* B0 dan B1, metrik, *raw result*, dan *manifest* | Hasil *compile*, *runtime*, dan memori dapat direproduksi dan *correctness* lulus |
| M8 CPU demonstration | Rangkaian CPU, program deterministik, serta jalur input dan tampilan | Program menjalankan *workload* yang ditentukan, dan hasil serta FPS aplikasi tercatat |
| M9 Stabilization and delivery | *Regression suite*, laporan kinerja, daftar keterbatasan, paket *build*, dan dokumentasi akhir | Tidak ada pemblokir *correctness* yang diketahui dan demo dapat diulang dari *setup* bersih |

M8 dapat memakai CPU sederhana terlebih dahulu. Snake atau Pong adalah target demonstrasi yang baik bila CPU, memori, dan jalur tampilan sudah cukup stabil. Bila belum, program deterministik yang lebih kecil tetap dapat membuktikan eksekusi CPU. Kegagalan mencapai Snake atau Pong tidak disamarkan sebagai keberhasilan, melainkan dilaporkan sebagai pengurangan scope yang eksplisit.

#### 3.6.2 Urutan Ketergantungan Pekerjaan

Urutan pada Tabel 3.14 ditentukan oleh ketergantungan teknis, risiko *correctness*, dan nilai demonstrasi. Pekerjaan *test fixture* dan dokumentasi dapat berjalan paralel. Editor dasar tidak harus menunggu seluruh *runtime* selesai, sehingga editor dan *engine* dikerjakan sebagai dua jalur yang bertemu pada *vertical slice*. Editor tidak boleh dibangun dengan struktur data sementara yang kelak memaksa *compiler* membaca objek UI.

**Tabel 3.14** Urutan ketergantungan pekerjaan

| Prioritas | Pekerjaan | Kondisi keluar |
| --- | --- | --- |
| P0-A | Pembekuan semantik dan *test fixture* | TBD semantik yang menghambat inti ditutup, dan *fixture* serta perilaku yang diharapkan tersedia |
| P0-B | Model domain dan dasar serialisasi | Rangkaian dapat dibuat, disimpan, dimuat, dan divalidasi |
| P0-C | Konektivitas dan inti semantik tanpa UI | *Fixture* rangkaian menghasilkan *netlist* dan diagnostik yang deterministik |
| P0-D | *Reference evaluator* dan *correctness harness* | Primitif dan *fixture* kombinasional dapat dibandingkan |
| P0-E | *Vertical slice runtime* terkompilasi | Input → *compile* → eksekusi → output, dengan *regression test* lulus |
| P0-F | *State* sekuensial dan memori | *Counter*, register, RAM, dan ROM lulus uji siklus demi siklus |
| P0-G | *Vertical slice* editor | Pengguna dapat membuat dan menjalankan rangkaian yang sama tanpa menyunting *file* manual |
| P0-H | *Worker*, kontrol, dan *observability* | *Pause*, *step*, *reset*, dan *probe* stabil, dan UI tidak membaca *runtime* yang dapat berubah secara langsung |
| P0-I | *Benchmark* dan *profiling* | *Baseline* waktu *compile*, CPS, memori, dan UI tersedia |
| P1-A | Hierarki, *chip* kustom, dan *source map* matang | *Instance* hierarkis yang berulang berperilaku independen |
| P1-B | *Workload* CPU dan validasi aplikasi | Program CPU deterministik menghasilkan output yang diharapkan |
| P1-C | Optimasi berbasis bukti | Perubahan lulus *correctness* dan menunjukkan manfaat pada *benchmark* |

Tabel 3.14 menunjukkan dua jalur yang bertemu pada *vertical slice*, yaitu jalur *engine* (P0-A sampai P0-F) dan jalur editor (P0-G). Keduanya kemudian disatukan oleh *worker* dan *observability* (P0-H) serta *benchmark* (P0-I). Pekerjaan P1 dikerjakan setelah inti P0 stabil, sehingga urutan ini sekaligus menjadi aturan prioritas bila waktu menyempit.

#### 3.6.3 Kriteria Keluar M0

M0 dinyatakan selesai bila kriteria pada Tabel 3.13 (keputusan semantik, ID kebutuhan, dan alat kerja yang tertulis) serta kriteria teknis berikut sama-sama terpenuhi, karena file manifes dan struktur folder saja tidak cukup. *Baseline* harus dibuktikan dengan *vertical slice* teknis berikut.

1. *Cargo workspace* dapat membangun `dlf-core` dan `dlf-godot` untuk target *x86\_64-pc-windows-msvc*.
2. Godot 4.7.2 dapat membuka proyek Windows dan memuat GDExtension yang dibangun dari Rust dengan godot-rust 0.5.5 dan *API level* 4.7.
3. GDScript dapat memanggil *facade* Rust dengan API bertipe dan menerima hasil yang benar.
4. *Worker* Rust dapat menerima perintah, mengubah *state* milik Rust, dan menerbitkan *snapshot* tanpa menyentuh API atau *scene tree* Godot.
5. *Main thread* Godot dapat mengambil *snapshot* dan memperbarui UI tanpa membagikan *runtime state* yang dapat berubah.
6. *Prototype canvas* terpusat dapat menggambar rangkaian besar, melakukan *pan/zoom*, *hit-testing*, dan *selection*, dengan hasil dicatat sebagai *baseline*.
7. *Cargo test* dan *smoke test* ekstensi Godot dalam mode *headless* dapat dijalankan ulang dari *clean checkout* di Windows.
8. *Release build* Windows dapat dihasilkan, sedangkan *installer* final menjadi pekerjaan M9.

Bila integrasi godot-rust, batas *thread*, atau *prototype canvas* gagal pada tahap ini, keputusan stack dievaluasi ulang sebelum M1 dan M5 bergantung terlalu jauh padanya. Pilihan teknologi tidak diperlakukan kebal revisi hanya karena sudah dicatat.

#### 3.6.4 Alur Kerja Build dan Tooling

- *Build*, *test*, *lint*, dan *benchmark* Rust dijalankan melalui Cargo. Proyek Godot dibuka melalui Godot Editor, dan CLI (command-line interface, antarmuka baris perintah) *headless* dipakai untuk *import*, *smoke test*, dan otomasi.
- Orkestrasi awal memakai skrip PowerShell agar sesuai dengan target Windows tanpa *task runner* tambahan. Docker tidak dipakai dalam alur kerja normal.
- *Debug build* dipakai untuk iterasi, sedangkan *release build* dipakai untuk *benchmark* kinerja dan pengemasan. Hasil *debug* dan *release* tidak dibandingkan seolah-olah setara.
- Dependensi baru harus memiliki alasan yang jelas, lisensi yang dapat diterima, aktivitas pemeliharaan yang memadai, dan biaya *build* atau *runtime* yang terukur.
- Pengemasan *installer* final dikerjakan pada M9 dan bukan penghambat inti *engine*.

### 3.7 Rencana Pengujian, Evaluasi, dan Kriteria Keberhasilan

Rencana ini menerapkan metode evaluasi yang telah dibahas pada subbab 2.4. *Correctness suite* menjadi gerbang wajib sebelum pengukuran kinerja dianggap valid, dan kinerja dilaporkan per aspek, bukan sebagai satu skor gabungan.

#### 3.7.1 Lapisan Pengujian

**Tabel 3.15** Lapisan pengujian dan *tooling*

| Lapisan | *Tool* atau alur kerja | Tujuan |
| --- | --- | --- |
| *Formatting* | `cargo fmt --check` | Konsistensi kode Rust |
| *Static lint* | `cargo clippy --workspace --all-targets` | Menemukan pola Rust yang berisiko atau tidak idiomatis |
| *Unit* dan *integration test* Rust | `cargo test --workspace` | Menguji model, konektivitas, *compiler*, IR, *runtime*, dan semantik tanpa Godot |
| *Smoke test* proyek Godot | *Import* dan pemuatan ekstensi mode *headless* | Memastikan proyek dapat dibuka dan ekstensi *native* termuat |
| Integrasi GDScript dan UI | *Test scene* dan *script* mode *headless* | Memvalidasi kontrak Godot dan Rust untuk *facade*, siklus hidup perintah, dan data presentasi |
| Uji visual dan manual | Godot Editor dengan *fixture* rangkaian terkontrol | Memvalidasi *zoom/pan*, *selection*, *wiring*, *hit-testing*, *inspector*, dan *rendering* |
| *Microbenchmark* Rust | Criterion untuk operasi inti yang terisolasi | Mengukur *compiler*, *runtime*, dan struktur data tanpa *rendering* |
| *Application benchmark* | *Test scene* Godot yang deterministik dengan *fixture* yang sama antarpengukuran | Mengukur *frame time* UI, latensi perintah, biaya *snapshot*, memori, dan perilaku *end-to-end* |
| CI | GitHub Actions dengan *runner* Windows x64 | Menjalankan *formatting*, *lint*, *test*, dan *smoke test* pada lingkungan yang bersih |

Tabel 3.15 dapat dibaca dalam empat kelompok. Format, *lint*, dan *unit test* menjaga kebenaran inti tanpa bergantung pada Godot. *Smoke test* dan uji integrasi GDScript memvalidasi kontrak antara Godot dan Rust. Uji visual manual melengkapi hal yang sulit diotomasi, dan dua *benchmark* memisahkan pengukuran inti dari pengukuran aplikasi agar biaya UI tidak tercampur dengan biaya simulasi. CI (continuous integration, yaitu layanan yang menjalankan pemeriksaan setiap kali kode berubah) menjalankan seluruh pemeriksaan otomatis pada lingkungan yang bersih.

#### 3.7.2 Workload Uji

**Tabel 3.16** *Workload suite*

| ID | *Workload* | Hal yang diuji |
| --- | --- | --- |
| W01 | Logika primitif | *Dispatch*, baca dan tulis sinyal, dan operasi dasar |
| W02 | *Fanout* dan propagasi | Biaya *fanout*, *schedule* berurutan level, dan propagasi perubahan |
| W03 | *Datapath* aritmetika | Operasi multi-bit, spesialisasi lebar, dan kedalaman ketergantungan |
| W04 | Bus dan konektivitas | Pemetaan bit, penyimpanan terkemas, waktu *compile*, dan memori |
| W05 | Logika sekuensial | *Current/next state*, *clock tick*, dan *simultaneous commit* |
| W06 | Subsistem memori | Semantik akses memori, *layout* penyimpanan, dan *throughput* baca dan tulis |
| W07 | *Feedback* dan SCC | Klasifikasi SCC, penyelesaian, dan deteksi osilasi |
| W08 | Rangkaian hierarkis | Elaborasi, *flattening*, spesialisasi, dan *source map* |
| W09 | Skala sintetis | Skalabilitas terhadap jumlah komponen, net, *fanout*, dan *state* |
| W10 | *Workload* CPU | *Throughput* siklus *end-to-end* serta interaksi memori dan kontrol |
| W11 | Tampilan interaktif | Integrasi I/O *host*, *batching*, *snapshot* UI, dan *rendering* |
| W12 | Demo aplikasi | Kinerja yang dapat dimainkan dan *frame pacing* dengan Snake atau Pong |

W01 sampai W09 adalah *workload* diagnostik, sedangkan W10 sampai W12 adalah *workload* sistem yang menunjukkan apakah optimasi tingkat rendah benar-benar membantu tujuan produk. W12 tidak boleh menjadi satu-satunya tolok ukur karena hasilnya mencampur biaya rangkaian CPU, protokol tampilan, input, dan *renderer*. Rangkaian standar ISCAS'85 dan ISCAS'89 menjadi sumber *workload* tambahan setelah format dan semantiknya dipetakan ke DLF, dan prioritas awalnya adalah subset kecil untuk skala kombinasional dan regresi sekuensial (Czech Technical University in Prague, n.d.). Daftar rangkaian final dipilih setelah kompatibilitas impor diuji.

#### 3.7.3 Baseline Perbandingan

**Tabel 3.17** *Baseline* evaluasi

| *Baseline* | Deskripsi | Dipakai untuk |
| --- | --- | --- |
| B0 *Semantic reference evaluator* | Evaluator sederhana dan mudah diperiksa, tanpa optimasi, dengan semantik yang sama | *Oracle correctness* dan pembanding biaya optimasi |
| B1 *Straightforward runtime* | *Runtime* awal yang mengeksekusi operasi terkompilasi secara langsung dengan struktur sederhana | *Baseline* internal sebelum optimasi *layout* dan *scheduling* |
| B2 *Candidate runtime variants* | Varian IR ringkas, *schedule* berurutan level, *layout* penyimpanan, *dispatch*, atau *batching* | Mengisolasi dampak setiap keputusan arsitektur |
| B3 *External simulator* | Simulator lain hanya bila input, *workload*, dan semantiknya dapat disejajarkan | Konteks eksternal, bukan klaim peringkat universal |
| B4 *End-to-end DLF* | *Compile*, *worker*, jembatan perintah, *snapshot*, dan *renderer* | Pengalaman aktual dan target produk |

B0 tidak harus cepat, tetapi harus transparan dan benar, serta tidak berbagi implementasi inti yang sama dengan kandidat yang diuji. Verilator hanya menjadi pembanding eksternal kontekstual untuk rangkaian yang dapat diekspor ke HDL dengan perilaku setara. Simulator visual lain tidak dijadikan *baseline* numerik utama kecuali *workload* identik dapat dijalankan, konfigurasi dikendalikan, dan definisi siklusnya sama. Klaim pemasaran atau angka *benchmark* tanpa *workload* dan metodologi yang diketahui tidak dipakai sebagai angka pembanding.

#### 3.7.4 Tahapan Eksekusi Evaluasi

Karena *runtime* DLF belum tersedia, validasi dilakukan bertahap.

1. **Fase A, *fixture* dan *oracle* semantik.** *Fixture* rangkaian kecil dibuat untuk primitif, kombinasional, sekuensial, memori, konektivitas, hierarki, dan *feedback*. B0 dibuat, dan perilaku yang diharapkan dikunci menurut profil semantik.
2. **Fase B, *microbenchmark compile*.** Snapshot, validasi, elaborasi hierarki, konektivitas, analisis SCC, *lowering*, dan alokasi diukur terpisah tanpa UI penuh.
3. **Fase C, *prototype runtime*.** B1 dan satu *runtime* kandidat yang ringkas diuji pada *workload* sintetis dan subset ISCAS, dengan *differential correctness* sebelum hasil kinerja diterima.
4. **Fase D, *worker*, *observability*, dan aplikasi.** *Command queue*, *batching*, *snapshot*, *probe*, dan *rendering* ditambahkan, lalu program CPU deterministik dijalankan, disusul demo Snake atau Pong bila semantik CPU, memori, dan tampilan cukup stabil.
5. **Fase E, *regression baseline*.** Hasil *correctness* dijadikan *regression suite*, *baseline* kinerja disimpan per *workload*, dan setiap optimasi baru harus lulus *correctness* serta dibandingkan dengan *baseline* yang relevan.

Setiap pengukuran mencatat *commit*, versi *toolchain* Rust dan Godot, profil *build*, CPU, RAM, GPU dan *driver* bila relevan, identitas *fixture*, *warm-up*, jumlah pengulangan, dan *raw result*. Waktu *compile*, *throughput* inti *runtime*, memori dan alokasi, *frame time* UI, latensi dari perintah hingga tampil, serta FPS aplikasi *end-to-end* dilaporkan terpisah.

**Perangkat uji.** Seluruh pengukuran dilakukan pada satu laptop, yaitu Lenovo Legion Y520 dengan prosesor Intel Core i7-7700HQ, RAM 16 GB, dan GPU NVIDIA GeForce GTX 1050 Ti. Karena berupa laptop, hasil pengukuran rentan terhadap penurunan frekuensi akibat suhu dan mode daya. Pengukuran karena itu dilakukan dengan adaptor daya terhubung dan profil daya yang sama, dan kondisi tersebut dicatat bersama versi Windows pada *manifest*. Angka dari perangkat ini tidak diklaim berlaku untuk perangkat lain.

#### 3.7.5 Kriteria Keberhasilan

Proyek dinyatakan berhasil bila deliverable berikut tersedia dan dapat direproduksi sesuai cakupan fitur yang telah diimplementasikan, dengan P0 sebagai syarat wajib dan P1 sebagai target.

1. *Workload suite* mencakup primitif, konektivitas, kombinasional, sekuensial, memori, hierarki, skala sintetis, CPU, dan tampilan interaktif.
2. B0 atau *oracle* yang setara tersedia untuk *differential validation* pada subset semantik yang telah didefinisikan.
3. *Correctness suite* mencakup tabel kebenaran, lebar bit, resolusi empat nilai dan net, *clock* dan *state*, memori, *feedback* dan SCC, konektivitas, hierarki, serta kesesuaian *compiler* dan *runtime* sesuai fitur yang telah diimplementasikan.
4. *Benchmark harness* menyimpan *manifest* dan *raw result* serta memisahkan waktu *compile*, *runtime* inti, UI, dan *end-to-end*.
5. *Baseline* internal dan sekurangnya satu kandidat *runtime* dapat dibandingkan pada *workload* yang sama.
6. Eksperimen yang relevan telah dijalankan atau secara eksplisit ditandai tidak relevan atau belum dapat diuji beserta alasannya.
7. Keputusan arsitektur yang berubah karena hasil eksperimen dicatat beserta bukti, *trade-off*, dan dampaknya.
8. Tidak ada klaim kinerja final yang hanya bersumber dari *microbenchmark*, dan *workload* CPU serta aplikasi dilaporkan bila tersedia.

Sebuah fitur atau milestone tidak dianggap selesai hanya karena dapat berjalan pada satu demo manual. *Definition of Done* mensyaratkan hal berikut.

- ***Behavior.*** Kebutuhan dan semantik yang relevan tertulis serta dapat diuji.
- ***Correctness.*** *Unit test*, *golden test*, dan *differential test* yang relevan lulus dan *regression suite* tidak menurun.
- ***Diagnostics.*** Masukan atau rangkaian tidak valid menghasilkan *error* yang dapat ditelusuri ke komponen, pin, *wire*, atau jalur hierarki.
- ***Reproducibility.*** Uji yang gagal menyimpan *fixture*, urutan input, *seed*, serta keluaran yang diharapkan dan yang aktual.
- ***Performance.*** Fitur yang menyentuh jalur panas atau sumber daya besar memiliki *benchmark* sebelum dan sesudah, dan optimasi tidak diterima hanya berdasarkan intuisi.
- ***Architecture.*** Kepemilikan Editor → *Snapshot* → *Artifact* → *Runtime State* tidak dilanggar.
- ***Documentation.*** Keputusan, keterbatasan, dan TBD diperbarui pada laporan atau *decision log*.
- ***Demo.*** Milestone dapat dijalankan dari *build* dan *setup* bersih dengan langkah yang terdokumentasi.

#### 3.7.6 Evaluasi Kebergunaan oleh Pengguna (Target P1)

Evaluasi ini memeriksa apakah kebutuhan yang diturunkan dari kajian benar-benar sesuai dengan pengguna sasaran. Ia tidak dimaksudkan sebagai uji statistik, melainkan sebagai masukan perbaikan UI dan bukti awal atas manfaat yang diklaim pada Bab I.

- **Peserta.** Lima sampai sepuluh mahasiswa atau peminat yang pernah mempelajari logika digital, dipilih secara sukarela. Data dicatat tanpa identitas.
- **Waktu.** Pada M6, setelah editor dan kontrol simulasi berjalan, dan diulang pada M8 bila demo CPU tercapai.
- **Tugas.** Menyusun *half adder* dari gerbang dasar lalu menjalankannya, memperbaiki rangkaian yang sengaja diberi kesalahan koneksi dengan bantuan diagnostik, dan, pada M8, menjalankan program contoh pada CPU yang sudah tersedia.
- **Ukuran.** Tingkat penyelesaian tugas, waktu penyelesaian, jumlah kesalahan, dan skor *System Usability Scale* (SUS), yaitu kuesioner sepuluh pernyataan berskala Likert lima poin untuk menilai kebergunaan secara umum (Brooke, 1996), ditambah komentar bebas.
- **Batasan.** Jumlah peserta kecil dan tidak mewakili populasi. Hasilnya dilaporkan apa adanya sebagai indikasi, dan tidak dipakai untuk klaim umum tentang efektivitas pembelajaran.

Bila waktu menyempit, evaluasi ini dikurangi lebih dahulu karena berstatus P1, dan keputusan tersebut dicatat pada *decision log*.

### 3.8 Manajemen Risiko

Risiko dinilai dengan *probability* (P) dan *impact* (I), masing-masing pada skala 1 sampai 5, dan *exposure* dihitung sebagai P dikali I. Nilai ini adalah penilaian awal untuk membantu penentuan prioritas, bukan probabilitas statistik. Setiap risiko ditulis sebagai kondisi beserta konsekuensinya, dan memiliki mitigasi, *trigger*, serta *fallback*. Pendekatan ini mengikuti praktik *continuous risk management* yang mencatat pengenal, prioritas, probabilitas, dampak, *exposure*, rentang waktu, dan tindakan mitigasi (NASA, n.d.-b). Risiko dengan *exposure* 15 sampai 25 menjadi perhatian milestone aktif dan ditinjau setiap kali milestone selesai atau *trigger* terjadi, sedangkan risiko dengan *exposure* 8 sampai 14 dipantau pada pekerjaan terkait. Skor tidak menggantikan pertimbangan rekayasa, dan satu risiko dengan dampak *correctness* tinggi tetap dapat menjadi pemblokir meskipun probabilitasnya rendah.

**Tabel 3.18** *Risk register*

| ID | Risiko dan konsekuensi | P | I | *Exposure* | Mitigasi dan *fallback* |
| --- | --- | --- | --- | --- | --- |
| R01 | Semantik belum cukup operasional sehingga *compiler* dan *runtime* menafsirkan perilaku secara berbeda dan *correctness* sulit dibuktikan | 4 | 5 | 20 | *Decision table* dan *executable test* dibuat sebelum fitur terkait diimplementasikan. *Fallback* membatasi struktur yang belum didefinisikan dengan diagnostik |
| R02 | Scope melebar ke *polish* editor, komponen, JIT, atau *plugin* sebelum *vertical slice* selesai | 4 | 5 | 20 | P0/P1/P2 dan *gate* milestone diterapkan. *Fallback* memindahkan pekerjaan baru ke P2 |
| R03 | Konektivitas dan hierarki terlalu kompleks sehingga terjadi *bug* pemetaan net atau biaya *compile* tinggi | 4 | 5 | 20 | *Fixture* dan *validator* tanpa UI dibangun, dan *junction*, persilangan, *splitter*, lebar, serta hierarki diuji terpisah. *Fallback* membatasi kedalaman hierarki dan fitur *wiring* serta menolak rangkaian tidak valid |
| R04 | Arsitektur *runtime* terlalu abstrak atau dioptimalkan terlalu dini sehingga *debugging* dan *correctness* melambat | 3 | 5 | 15 | Dimulai dari B0 dan B1 yang sederhana, dan IR ringkas baru dipakai setelah kontrak stabil. *Fallback* memakai *interpreter* sederhana yang benar sebagai *baseline* |
| R05 | Kinerja tidak mencapai target CPU atau permainan walaupun *microbenchmark* terlihat cepat | 4 | 4 | 16 | *Workload* CPU *end-to-end* diukur, bukan CPS saja. *Fallback* menurunkan kompleksitas demo CPU, bukan menyamakan CPS dengan FPS |
| R06 | Logika empat nilai, *multi-driver*, dan *feedback* atau SCC menambah kompleksitas lebih besar dari perkiraan | 4 | 5 | 20 | Semantik dikunci per *construct* dan subset yang teruji diimplementasikan. *Fallback* mendukung rangkaian tanpa siklus dan subset *feedback* yang terdefinisi, sedangkan rangkaian lain ditolak dengan diagnostik |
| R07 | Skema serialisasi berubah saat model berkembang sehingga *fixture* proyek tidak kompatibel | 3 | 4 | 12 | *Stable ID*, versi skema, *migration test*, dan versi *fixture*. *Fallback* menyediakan konverter eksplisit untuk versi yang dipakai selama proyek |
| R08 | Sinkronisasi *worker* dan UI menimbulkan *race*, *snapshot* basi, atau latensi perintah | 3 | 4 | 12 | *Worker* memiliki *Runtime State* secara eksklusif, memakai perintah dan *snapshot* yang *immutable*, serta diuji dengan *stress test*. *Fallback* menjalankan mode sinkron satu *thread* untuk *correctness* lebih dahulu |
| R09 | *Benchmark* tidak dapat direproduksi akibat *thermal drift*, *tooling*, atau *workload* yang tidak representatif | 4 | 3 | 12 | *Manifest*, pengulangan, *release build*, *raw result*, dan *workload* mikro serta aplikasi. *Fallback* melaporkan variansi dan membatasi klaim |
| R10 | Sumber daya pengembang atau waktu akademik tidak cukup untuk seluruh P0 dan P1 | 4 | 5 | 20 | Milestone berbasis kemampuan, cadangan waktu *debugging*, dan peninjauan scope berkala. *Fallback* mempertahankan simulator terkompilasi tanpa UI, *correctness*, serta *baseline* sekuensial dan memori, dengan mengurangi *polish* editor, hierarki lanjutan, dan demo permainan |
| R11 | Demo CPU terlalu ambisius karena ISA, peta memori, protokol tampilan, atau program tidak dibatasi | 3 | 4 | 12 | Subset ISA (instruction set architecture, yaitu daftar instruksi CPU) dan program deterministik ditetapkan, dan simulator dipisahkan dari *assembler*. *Fallback* memakai ISA mainan atau *datapath* yang lebih kecil dengan alasan yang dicatat |
| R12 | Terlalu banyak *test* dan *trace* menghambat kinerja dan UI atau membuat *test harness* mahal dirawat | 3 | 3 | 9 | *Fast correctness suite*, *extended suite*, dan mode *profiling* dipisah, serta *probe* terpilih dipakai di UI. *Fallback* memakai *trace sampling* dan hanya menyimpan kasus gagal |

Tabel 3.18 menunjukkan bahwa risiko terbesar bersumber dari tiga hal, yaitu semantik yang belum operasional (R01 dan R06), kompleksitas konektivitas dan hierarki (R03), serta kapasitas satu pengembang terhadap scope (R02 dan R10), masing-masing dengan *exposure* 20. Risiko arsitektur *runtime* dan kinerja (R04 dan R05) berada sedikit di bawahnya. Pola ini sejalan dengan prinsip proyek bahwa *correctness* dan pembatasan scope didahulukan dari optimasi.

#### 3.8.1 Strategi Fallback

Fallback harus mempertahankan integritas produk, sehingga pemangkasan dilakukan dari fitur yang menambah keluasan, bukan dari *correctness* inti.

**Tabel 3.19** Strategi *fallback* dan aturan pengurangan scope

| Jika terjadi kendala | Dipertahankan | Dipangkas atau di-*fallback* |
| --- | --- | --- |
| Waktu implementasi menyusut | Model rangkaian tanpa UI, *compile* dan *runtime*, semantik deterministik, *correctness suite* | *Polish* editor, alat *selection* lanjutan, tema, dan animasi |
| Konektivitas belum stabil | Perilaku *endpoint* dan *junction* yang kanonis serta diagnostik | Persilangan *wire*, *tunnel*, atau fitur perutean yang belum teruji |
| *Solver* SCC belum siap | Penjadwalan kombinasional tanpa siklus dan batas sekuensial yang eksplisit | *Feedback* kombinasional yang tidak didukung ditolak dengan *error compile*, tanpa hasil perkiraan |
| Logika empat nilai terlalu mahal | Definisi profil semantik dan cakupan *test* untuk *construct* yang didukung | Kombinasi komponen dan *driver* dikurangi, dan X atau Z tidak diam-diam diubah menjadi 0 atau 1 |
| Kinerja *runtime* rendah | *Correctness*, *profiler*, *baseline* B1, dan *workload* yang dapat direproduksi | JIT ditunda, demo CPU disederhanakan, dan optimasi hanya untuk *bottleneck* yang terukur |
| UI dan *worker* belum stabil | Mode *runtime* sinkron yang dapat diuji dan API *controller* | *Worker thread* dan penggabungan *snapshot* ditunda hingga *correctness* *runtime* stabil |
| Hierarki terlalu kompleks | Rangkaian primitif dan *source mapping* minimum | *Nesting* dan parameterisasi *chip* dibatasi, atau rangkaian datar dipakai untuk demo |
| Snake atau Pong belum tercapai | Program uji CPU yang deterministik dengan output terukur | Demo diturunkan ke *counter*, uji memori, program subset ISA, atau uji tampilan, dan target yang tidak tercapai dilaporkan |

Satu *fallback* tidak dapat ditawar. Simulasi tidak boleh ditampilkan seolah-olah valid bila rangkaian memiliki semantik yang belum didukung. *Compiler* harus menolak rangkaian tersebut atau memberi diagnostik yang jelas.

### 3.9 Keterlacakan Kebutuhan

Tabel 3.20 menghubungkan setiap kebutuhan fungsional P0 dengan tujuan Tugas Akhir, milestone yang menyelesaikannya, dan bukti penerimaan yang akan diperiksa. Dengan tabel ini, kemajuan proyek dapat diukur dari bukti yang tersedia, bukan dari anggapan bahwa sebuah fitur sudah selesai.

**Tabel 3.20** Keterlacakan kebutuhan, tujuan, milestone, dan bukti

| Kebutuhan | Tujuan | Milestone | Bukti penerimaan |
| --- | --- | --- | --- |
| KF-01 Dokumen rangkaian | 4 | M1 | *Round-trip fixture* dan uji *structural validator* |
| KF-02 Editor dasar | 4 | M5 | *Test scene* mode *headless* dan uji visual manual |
| KF-03 *Library* primitif | 2 | M2 dan M3 | *Golden test* tabel kebenaran primitif |
| KF-04 Konektivitas | 1 | M2 | *Fixture* konektivitas dan pemetaan net yang diharapkan |
| KF-05 Pipeline *compile* | 1 | M3 | Uji determinisme *compile* dan uji diagnostik |
| KF-06 *Runtime* | 1 dan 3 | M3 | *Differential test* terhadap B0 |
| KF-07 Sekuensial | 2 dan 3 | M4 | Uji *counter* dan *shift register* |
| KF-08 Memori | 2 | M4 | Uji baca, tulis, dan inisialisasi memori |
| KF-09 Kontrol simulasi dan *worker* | 3 | M6 | Uji urutan perintah dan *stress test* *worker* |
| KF-10 *Observability* | 3 dan 5 | M6 | Uji *probe* dan *snapshot* |
| KF-11 Validasi | 5 | M3 sampai M9 | *Regression suite* yang tidak menurun |
| KF-12 *Benchmark* | 5 | M7 | *Manifest* dan *raw result* |
| Target P1, demo CPU | 6 | M8 | Program CPU deterministik menghasilkan output yang diharapkan dan FPS aplikasi tercatat |

Baris terakhir adalah target P1, sehingga ketiadaan buktinya dilaporkan sebagai pengurangan scope yang eksplisit, bukan sebagai kegagalan inti.

### 3.10 Spesifikasi Antarmuka dan Kontrak Data

Subbab ini merinci kontrak yang dibutuhkan agar sistem pada subbab 3.3 sampai 3.5 dapat dibangun tanpa menebak. Setiap spesifikasi diberi status. Status *Baseline* berarti sudah diputuskan pada kajian arsitektur. Status *Usulan* berarti nilai bawaan yang diajukan penulis karena kajian belum memutuskannya, dan harus ditetapkan pada *decision log* sebelum milestone yang disebutkan. Spesifikasi berstatus Usulan tidak boleh dianggap final sebelum keputusan itu dicatat.

#### 3.10.1 Dokumen Rangkaian dan Identitas Objek

**Status.** Struktur entitas adalah *Baseline*. Format *file* dan aturan pembuatan ID adalah *Usulan* yang ditetapkan sebelum M1.

Dokumen rangkaian (*Authoring Document*) terdiri atas entitas pada Tabel 3.21. Dokumen tidak menyimpan *net* maupun *runtime state*, karena keduanya diturunkan atau dibuat saat *compile* dan simulasi.

**Tabel 3.21** Entitas dokumen rangkaian

| Entitas | Atribut utama | Keterangan |
| --- | --- | --- |
| *Project* | Daftar definisi rangkaian, versi skema, dan dependensi definisi komponen | Akar penyimpanan satu proyek |
| *CircuitDefinition* | ID, nama, antarmuka (*port* masuk dan keluar), serta daftar *instance*, *wire*, *junction*, dan *tunnel* | Satu definisi per *chip*, dan rangkaian utama juga merupakan satu definisi |
| *ComponentInstance* | ID *instance*, ID definisi beserta versi skema, nilai parameter, koneksi pin, transformasi visual, dan label | Tidak menyimpan *runtime state* |
| *WireSegment* | ID, dua *endpoint*, dan rute geometri | Objek visual, bukan *net* |
| *Junction* | ID, koordinat, dan cabang yang bertemu | Titik sambung topologis |
| *Tunnel* | ID, nama, dan *scope* | Menghubungkan *tunnel* bernama sama dalam *scope* yang sama |

Tabel 3.21 menunjukkan pemisahan antara apa yang digambar dan apa yang dihitung. *WireSegment* hanya membawa geometri, sedangkan konektivitas listrik baru muncul sebagai *net* saat *compile*. *Splitter* dan *merger* bukan entitas tersendiri, melainkan *ComponentInstance* dengan pemetaan bit yang eksplisit.

ID *instance*, pin, *wire*, *junction*, dan *tunnel* harus stabil melintasi pemindahan objek, *undo/redo*, serta *save/load* selama objek yang sama masih ada, dan tidak boleh bergantung pada koordinat atau urutan *array* (*Baseline*). Usulan untuk aturan pembuatannya adalah bilangan bulat 64-bit yang diberikan oleh penghitung milik *Project* dan tidak dipakai ulang dalam satu proyek. Format *file* yang diusulkan berupa teks berversi, misalnya JSON, karena mudah diperiksa dan dibandingkan pada sistem kontrol versi. Akar *file* memuat versi skema, definisi komponen bawaan dirujuk melalui ID dan versi (tidak disalin), sedangkan *chip* buatan pengguna disimpan di dalam proyek yang sama. Migrasi antarversi harus eksplisit dan menghasilkan diagnostik bila skema lama tidak kompatibel. Keputusan akhir mengenai penyematan definisi komponen pada *file* proyek atau pengelolaannya sebagai dependensi *library* berversi ditutup sebelum M1.

#### 3.10.2 Kontrak Definisi Komponen

**Status.** *Baseline*.

Setiap komponen mendeklarasikan kontraknya melalui bidang pada Tabel 3.22. Pengenal komponen tidak boleh berupa nama tampilan, karena dua *library* dapat memiliki komponen bernama sama.

**Tabel 3.22** Bidang kontrak definisi komponen

| Bidang | Fungsi |
| --- | --- |
| `definition_id` | Identitas stabil yang tidak bergantung pada nama tampilan |
| `schema_version` | Versi kontrak komponen, dipakai untuk migrasi proyek |
| `display_name`, `category`, `icon` | Metadata untuk *library* dan editor |
| `pins[]` | Antarmuka pin beserta arah, lebar, dan aturan koneksi |
| `parameters[]` | Parameter beserta tipe, nilai bawaan, dan batas |
| `behavior_kind` | Kombinasional, sekuensial, memori, struktural, atau I/O |
| `implementation_kind` | Primitif *native*, rangkaian komposit, atau adaptor eksternal |
| `state_schema` | Deklarasi *state* yang dimiliki *instance*, bila ada |
| `compiler_lowering` | Aturan penerjemahan ke IR dan *runtime* |
| `editor_metadata` | Simbol, label, tata letak pin, dan properti *inspector* |
| `diagnostics` | Kondisi tidak valid, peringatan, dan pesan yang dapat ditampilkan |

Kontrak pada Tabel 3.22 sengaja memisahkan metadata editor dari aturan *compile*, sehingga mengubah ikon atau posisi pin tidak memengaruhi perilaku simulasi.

**Kontrak pin.** Setiap pin memiliki ID pin yang stabil, nama tampilan, arah (*input*, *output*, atau *inout*), ekspresi lebar, urutan bit, dan peran logika (*clock*, *reset*, *enable*, *address*, *data*, atau sinyal biasa). Lebar dapat berupa konstanta atau ekspresi dari parameter, misalnya `data_width`. Pin masukan menyatakan apakah koneksinya wajib, boleh *floating*, atau memiliki nilai bawaan, dan nilai bawaan tidak boleh menjadi cara tersembunyi untuk mengubah X menjadi 0. Pin keluaran menyatakan apakah ia keluaran biasa dengan satu *driver* atau dapat menghasilkan Z, dan *inout* hanya dipakai pada komponen yang mendukung semantik *net* yang diresolusi.

**Kontrak parameter.** Parameter bertipe *boolean*, *integer*, *enum*, *bit-vector*, atau rujukan sumber daya (*file*). Parameter struktural, yaitu yang mengubah jumlah pin, lebar, atau struktur *runtime*, mengharuskan *compile* ulang. Parameter presentasi dapat berubah tanpa membangun ulang *runtime*. Parameter yang mengubah perilaku *runtime* disimpan dalam deskriptor hasil *compile* dengan invalidasi yang eksplisit.

#### 3.10.3 Spesifikasi Komponen P0

**Status.** *Usulan*, ditetapkan sebelum M3. Butir yang bergantung pada TBD ditandai.

Tabel 3.23 merinci pin dan perilaku komponen yang dibutuhkan P0. Tabel 3.24 melengkapinya dengan tabel kebenaran dasar untuk nilai 0, 1, dan X, dengan Z pada masukan gerbang biasa diperlakukan sebagai X.

**Tabel 3.23** Pin dan perilaku komponen P0

| Komponen | Pin (arah, lebar) | Perilaku |
| --- | --- | --- |
| *NOT* dan *buffer* | masukan `a[W]`, keluaran `y[W]` | `y` adalah negasi bit demi bit (NOT) atau salinan `a` (*buffer*) |
| AND, OR, XOR, NAND, NOR, XNOR | masukan `a[W]` dan `b[W]` (jumlah masukan menjadi parameter, bawaan 2), keluaran `y[W]` | Operasi bit demi bit sesuai Tabel 3.24 |
| *Tri-state buffer* | masukan `a[W]` dan `en[1]`, keluaran `y[W]` | `en` = 1 meneruskan `a`, `en` = 0 menghasilkan Z, dan `en` = X menghasilkan X |
| *Constant* | keluaran `y[W]` | Nilai tetap dari parameter |
| Pin masukan dan keluaran rangkaian | satu pin dengan lebar `W` | Titik antarmuka rangkaian atau *chip* |
| Sumber *clock* | keluaran `clk[1]` | Menghasilkan satu *active edge* per siklus sesuai konfigurasi |
| *Multiplexer* | masukan `d0` hingga `d(n-1)`, masing-masing `[W]`, dan `sel[ceil(log2 n)]`, keluaran `y[W]` | `y` = `d[sel]`. Perilaku bila `sel` bernilai X atau Z ditetapkan pada TBD-1 (usulan, `y` = X kecuali semua masukan sama) |
| *Splitter* dan *merger* | satu bus `[W]` ke beberapa irisan atau sebaliknya | Hanya pemetaan bit yang eksplisit, lebar divalidasi, tanpa logika |
| Register | masukan `d[W]`, `clk[1]`, `en[1]` opsional, dan `rst[1]` opsional, keluaran `q[W]` | Pada *active edge*, `q` menerima `d` bila `en` aktif, dan `rst` sinkron mengatur `q` menjadi 0. Prioritas `rst` terhadap `en` ditetapkan pada TBD-5, dan *state* awal pada TBD-4 |
| ROM | masukan `addr[A]`, keluaran `data[D]` | Pembacaan asinkron dari isi *image*, dan alamat di luar jangkauan atau bernilai X menghasilkan X |
| RAM satu *port* sinkron | masukan `addr[A]`, `din[D]`, `we[1]`, dan `clk[1]`, keluaran `dout[D]` | Pada *active edge*, `we` aktif menulis `din`, dan `dout` menerima isi lama (*read-first*). Tulis dengan alamat X ditetapkan pada TBD-6 |

**Tabel 3.24** Tabel kebenaran dasar empat nilai (Z pada masukan diperlakukan sebagai X)

| a | b | AND | OR | XOR |
| --- | --- | --- | --- | --- |
| 0 | 0 | 0 | 0 | 0 |
| 0 | 1 | 0 | 1 | 1 |
| 0 | X | 0 | X | X |
| 1 | 1 | 1 | 1 | 0 |
| 1 | X | X | 1 | X |
| X | X | X | X | X |

Pada Tabel 3.24, kombinasi yang simetris (misalnya a = X dan b = 0) bernilai sama dengan baris yang tertera. NOT menghasilkan 1 untuk 0, 0 untuk 1, dan X untuk X. NAND, NOR, dan XNOR adalah negasi dari AND, OR, dan XOR. Aturan ini konservatif, yaitu hasil hanya pasti bila masukan yang diketahui sudah menentukannya, sehingga ketidaktahuan tidak berubah diam-diam menjadi 0 atau 1. Bila X dan Z belum diimplementasikan pada P0, *runtime* memakai dua nilai (0 dan 1) untuk rangkaian yang seluruh *state* awalnya terdefinisi, dan kolom X berlaku sejak target P1 empat nilai.

#### 3.10.4 Representasi Runtime dan IR

**Status.** Struktur dan prinsip penyimpanan adalah *Baseline*. Daftar operasi awal dan pengodean empat nilai adalah *Usulan*, ditetapkan sebelum M3 (TBD-2).

*Runtime* tidak menjalankan objek per gerbang. Rangkaian diterjemahkan menjadi daftar rekaman operasi pada *array* yang berurutan di memori dan disusun menurut level ketergantungan. Setiap rekaman memiliki bidang pada Tabel 3.25.

**Tabel 3.25** Rekaman operasi IR

| Bidang | Fungsi |
| --- | --- |
| `opcode` | Jenis operasi |
| `dst` | Indeks *slot* sinyal tujuan |
| `src_a`, `src_b`, `src_c` | Indeks *slot* sumber, sebanyak yang dibutuhkan *opcode* |
| `width` | Lebar bit operasi, 1 hingga 64 |
| `flags` | Penanda tambahan, misalnya sinyal yang diamati sehingga tidak boleh dihapus oleh optimasi |

Sebagai contoh, rangkaian yang memasukkan A dan B ke gerbang AND lalu hasilnya ke XOR bersama C diterjemahkan menjadi dua rekaman berikut.

```
slot 0 = A, slot 1 = B, slot 2 = C, slot 3 = hasil AND, slot 4 = hasil XOR
OP_AND  dst=3  a=0  b=1  width=1
OP_XOR  dst=4  a=3  b=2  width=1
```

Operasi awal yang diusulkan adalah BUF, NOT, AND, OR, XOR, NAND, NOR, XNOR, CONST, MUX, SLICE, CONCAT, TRI (untuk *tri-state*, target P1), DFF\_NEXT (menghitung *next state* register), MEM\_READ, dan MEM\_WRITE. Seluruh data yang dibutuhkan *runtime* disimpan pada *array* kontigu pada Tabel 3.26.

**Tabel 3.26** *Array* data *runtime*

| *Array* | Isi | Dipakai oleh |
| --- | --- | --- |
| `SignalValue` | Nilai seluruh sinyal | Semua operasi |
| `Operation` | Rekaman IR berurutan menurut level | Evaluator |
| `LevelRange` | Batas awal dan akhir setiap level pada `Operation` | *Scheduler* |
| `StateSlot` | *State* saat ini dan berikutnya untuk register dan *latch* | Evaluasi sekuensial dan *commit* |
| `MemoryRegion` | Isi RAM dan ROM beserta lebar dan jumlah kata | MEM\_READ dan MEM\_WRITE |
| `InputBinding` | Pemetaan masukan *runtime* (*switch*, tombol, *keyboard*) ke *slot* | Penerapan perintah SetInput |
| `OutputBinding` | Pemetaan *slot* ke keluaran yang ditampilkan (LED, *display*) dan *probe* | Penerbitan *snapshot* |
| `SourceMap` | Pemetaan operasi dan *slot* ke jalur hierarki serta ID objek editor | Diagnostik dan *debugger* |

Pemisahan pada Tabel 3.26 membuat jalur eksekusi hanya mengakses *array* dengan indeks, tanpa mencari berdasarkan nama atau identitas objek. `SourceMap` adalah satu-satunya penghubung kembali ke objek editor, dan ia hanya dibaca saat menampilkan diagnostik.

Bus hingga 64 bit disimpan dalam satu *machine word* 64-bit, dan *width mask* diterapkan saat menulis agar bit di luar lebar bus tidak bocor (*Baseline*). Untuk pengodean empat nilai, diusulkan dua kandidat. Kandidat A memakai dua *word* per sinyal, yaitu *value* dan *unknown mask*, dengan Z dibedakan oleh *mask* tambahan hanya pada *net* yang diresolusi. Kandidat B memakai dua bit terkemas untuk setiap bit sinyal. Pilihan ditentukan melalui eksperimen pada M3, dan P0 boleh memakai dua nilai terlebih dahulu.

#### 3.10.5 Definisi Siklus dan Kontrol Eksekusi

**Status.** *Baseline*, kecuali nilai bawaan ukuran *batch* dan *state* awal pada P0 yang berstatus *Usulan*.

Satu siklus logis (*logical cycle*) adalah satu transisi *active edge* pada domain *clock* yang dijalankan. Siklus ini bukan simulasi waktu fisik dan tidak menyiratkan *propagation delay*. Urutan lengkap satu *active edge* adalah sebagai berikut, dan melengkapi daftar pada subbab 3.4.4 dengan penyelesaian ulang logika kombinasional setelah *commit*.

1. Menerapkan masukan eksternal yang telah diantrekan untuk batas ini.
2. Menyelesaikan logika kombinasional berdasarkan *current state*.
3. Mengambil seluruh masukan D atau *next state* dari register dan komponen sekuensial.
4. Mengevaluasi *reset*, *enable*, dan kendali memori sinkron sesuai prioritas komponen.
5. Menyimpan seluruh *next state* secara serentak (*simultaneous commit*).
6. Menyelesaikan ulang logika kombinasional agar keluaran mencerminkan *state* baru.
7. Menerbitkan *snapshot* keluaran dan *probe* sesuai kebijakan pencuplikan.

Semua elemen sekuensial membaca *current state* yang sama sebelum *commit*, sehingga register A tidak dapat berubah lebih dahulu lalu memengaruhi register B pada *edge* yang sama. Kendali eksekusi mengikuti aturan berikut.

- **Step.** Satu langkah *step* menjalankan satu siklus logis. Langkah setengah siklus hanya menjadi kontrol *debugger*, sedangkan semantik internal tetap mendefinisikan kapan *edge* terjadi dan kapan *state* di-*commit*.
- **Run.** *Worker* menjalankan siklus dalam *batch*. Ukuran *batch* tidak boleh mengubah hasil semantik. Nilai bawaannya ditentukan dari pengukuran pada M7, dan sebelum itu dipakai nilai tetap yang dapat diatur melalui konfigurasi (*Usulan*).
- **Sumber *clock*.** Sumber *clock* adalah sinyal masukan yang berubah menurut konfigurasi. Ia bukan alasan untuk menjalankan satu pembaruan per *frame* UI.
- ***Reset*.** *Reset* sinkron hanya diperiksa pada *active edge*, sedangkan *reset* asinkron mengubah *state* segera tanpa menunggu *clock*. Prioritas *reset* dan *enable* ditetapkan per komponen (TBD-5).
- ***State* awal.** *State* sekuensial yang belum diinisialisasi bernilai X pada mode ketat. Selama P0 memakai dua nilai, *state* awal bernilai 0 dan kebijakan ini dicatat pada laporan (*Usulan*, TBD-4).

#### 3.10.6 Protokol Worker dan UI

**Status.** Prinsip komunikasi adalah *Baseline* (subbab 3.4.4). Jenis perintah dan isi *snapshot* adalah *Usulan*, ditetapkan sebelum M6.

*Simulation worker* berada pada salah satu status pada Tabel 3.27, dan setiap status hanya menerima perintah tertentu.

**Tabel 3.27** Status *worker* dan perintah yang diterima

| Status | Arti | Perintah yang diterima |
| --- | --- | --- |
| *Idle* | *Worker* dibuat dan belum memuat artefak | Start |
| *Paused* | Artefak dimuat dan siklus berhenti pada batas siklus | Run, Step, Reset, Stop, SetInput, SetProbes, SetBatchSize |
| *Running* | Menjalankan siklus dalam *batch* | Pause, Stop, SetInput, SetProbes, SetBatchSize |
| *Error* | Kegagalan *runtime* tercatat dan *state* terakhir yang konsisten dipertahankan | Reset, Stop |
| *Stopped* | *Worker* dilepas dan *runtime state* dibuang | Tidak ada |

Tabel 3.27 menunjukkan bahwa satu-satunya cara menghentikan eksekusi yang sedang berjalan adalah Pause atau Stop, dan bahwa kegagalan *runtime* tidak melanjutkan simulasi secara diam-diam. Perintah yang diterima dan waktu penerapannya didefinisikan pada Tabel 3.28.

**Tabel 3.28** Perintah *worker*

| Perintah | Parameter | Efek | Diterapkan pada |
| --- | --- | --- | --- |
| Start | Handle artefak | Membuat *Runtime Instance* dan mengubah status dari *Idle* ke *Paused* | Seketika |
| Run | Tidak ada | *Paused* menjadi *Running* | Batas *batch* |
| Pause | Tidak ada | *Running* menjadi *Paused* | Batas siklus |
| Step | Jumlah siklus n (bawaan 1) | Menjalankan n siklus lalu berhenti pada *Paused* | Batas siklus |
| Reset | Tidak ada | Mengembalikan *state* ke *state* awal dan penghitung siklus ke 0 | Batas siklus |
| Stop | Tidak ada | Menghentikan *worker* dan membuang *state* | Batas siklus |
| SetInput | ID *binding* dan nilai | Mengubah masukan *runtime* | Batas siklus berikutnya, dengan urutan terjamin |
| SetProbes | Daftar ID *probe* | Memilih sinyal yang dipublikasikan | Batas *batch* |
| SetBatchSize | Jumlah siklus | Mengubah ukuran *batch* | Batas *batch* |

Setiap *snapshot* yang diterbitkan *worker* memuat bidang pada Tabel 3.29.

**Tabel 3.29** Isi *snapshot* keluaran

| Bidang | Isi |
| --- | --- |
| `sequence` | Nomor urut *snapshot*, naik secara monoton |
| `cycle_count` | Jumlah siklus sejak Start atau Reset |
| `status` | Status *worker* saat *snapshot* dibuat |
| `outputs` | Nilai seluruh `OutputBinding` yang ditampilkan |
| `probes` | Nilai *probe* yang dipilih |
| `speed` | Siklus per detik yang terukur pada rentang terakhir |
| `diagnostics` | Diagnostik *runtime*, bila ada |

Kebijakan penerbitan adalah sebagai berikut. *Snapshot* diterbitkan paling banyak satu kali per interval presentasi, dan usulannya sekali per *frame* UI. UI mengambil *snapshot* terbaru, dan *snapshot* lama boleh dilewati atau digabung bila UI tertinggal. *Worker* tidak pernah menunggu UI. *Trace* yang harus lengkap memakai jalur terpisah berupa *ring buffer* berkapasitas tetap, dan bila penuh, penangkapan dihentikan serta ditandai tidak lengkap, bukan memperlambat *worker* (*Usulan*). Perintah diterapkan menurut urutan kedatangannya pada batas yang ditetapkan Tabel 3.28.

#### 3.10.7 API Facade Godot dan Rust

**Status.** Prinsip adalah *Baseline*. Daftar fungsi adalah *Usulan*, ditetapkan pada M0 dan dimatangkan pada M5 dan M6.

*Facade* adalah satu-satunya pintu antara GDScript dan Rust. Aturannya adalah sebagai berikut. API bertipe dan berukuran besar (*coarse-grained*), perubahan yang secara semantik satu transaksi dikirim sebagai satu *batch*, data besar memakai *typed array* atau *packed array*, *worker* tidak pernah memanggil objek Godot, dan seluruh balasan dari *worker* diambil UI melalui pemanggilan *poll* pada *main thread*. Fungsi yang diusulkan tercantum pada Tabel 3.30.

**Tabel 3.30** Fungsi *facade* yang diusulkan

| Fungsi | Masukan | Keluaran | Catatan |
| --- | --- | --- | --- |
| `open_project` | Path *file* | Handle proyek atau diagnostik | Memuat dan memigrasi dokumen |
| `save_project` | Handle, path | Status | Menyimpan dokumen |
| `get_library` | Tidak ada | Daftar definisi komponen | Untuk palet komponen |
| `apply_edit` | Handle, *batch* operasi | Hasil per operasi (ID baru atau *error*) | Satu transaksi menjadi satu entri *undo* |
| `undo`, `redo` | Handle | Daftar objek yang berubah | Memulihkan dokumen dan konektivitas yang sama |
| `compile` | Handle | Handle artefak atau daftar diagnostik | Tidak memulai *worker* |
| `worker_start` | Handle artefak | Status | Mengirim perintah Start |
| `worker_command` | Perintah pada Tabel 3.28 | Diterima atau ditolak sesuai Tabel 3.27 | Tidak menunggu simulasi selesai |
| `poll_snapshot` | Tidak ada | *Snapshot* terbaru atau kosong | Dipanggil dari *main thread* |
| `shutdown` | Tidak ada | Tidak ada | Menghentikan *worker* dan melepaskan sumber daya |

Tabel 3.30 memastikan bahwa UI tidak pernah menunggu simulasi. Pemanggilan yang berpotensi lama, seperti `compile`, dijalankan sedemikian rupa sehingga *main thread* tetap dapat menggambar, dan hasilnya diambil kemudian.

#### 3.10.8 Diagnostik

**Status.** Struktur adalah *Usulan*, dan kategori kondisi adalah *Baseline*.

Setiap diagnostik memiliki bidang berikut. `severity` bernilai *error*, *warning*, atau *info*. `code` adalah string stabil seperti `CONN-WIDTH-MISMATCH`. `message` adalah teks untuk pengguna. `stage` menyatakan tahap *compile* atau *runtime* tempat diagnostik muncul. `primary` merujuk objek utama melalui ID *instance*, pin, *wire*, atau jalur hierarki, dan `related` memuat rujukan objek lain yang terlibat. *Error* menghalangi pembuatan artefak, sedangkan *warning* tidak. Kondisi yang wajib dideteksi tercantum pada Tabel 3.31.

**Tabel 3.31** Kondisi diagnostik wajib

| Kondisi | Objek yang disorot | Contoh kode |
| --- | --- | --- |
| Pin masukan wajib tidak terhubung | Pin terminal | `CONN-PIN-UNCONNECTED` |
| *Wire* menggantung | *Endpoint* atau segmen | `CONN-WIRE-DANGLING` |
| Lebar bus tidak cocok | Pin dan segmen yang terlibat | `CONN-WIDTH-MISMATCH` |
| Beberapa *driver* pada *net* yang tidak diresolusi | Seluruh *driver* dan *net* | `NET-MULTI-DRIVER` |
| *Tunnel* tanpa pasangan | *Tunnel* dan *scope* | `CONN-TUNNEL-UNPAIRED` |
| Pemetaan *splitter* atau *merger* tidak valid | Komponen dan bit atau cabang | `CONN-SPLIT-INVALID` |
| *Net* tanpa *driver* | *Net* dan konsumennya | `NET-UNDRIVEN` |
| *Chip* memanggil dirinya sendiri | Rantai instansiasi | `HIER-RECURSION` |
| *Feedback* tidak konvergen | Wilayah SCC | `DEP-SCC-OSCILLATION` |

Setiap baris pada Tabel 3.31 menunjuk objek yang dapat disorot di *canvas*. Dengan demikian, pesan seperti "lebar bus tidak cocok" selalu dapat ditelusuri ke pin dan segmen yang bersangkutan, sebagaimana dipersyaratkan KNF-03.

#### 3.10.9 Operasi Penyuntingan, Undo/Redo, dan Invalidasi

**Status.** *Baseline*, kecuali mekanisme penyimpanan riwayat yang berstatus *Usulan*.

Satu aksi pengguna yang secara semantik merupakan satu kesatuan, misalnya *paste*, pemindahan banyak objek sekaligus, atau pembuatan sekumpulan komponen dan koneksi, adalah satu transaksi. Satu transaksi menjadi satu entri *undo*. *Undo* dan *redo* harus memulihkan dokumen sehingga konektivitas hasilnya identik dan ID objek tidak berubah. Riwayat disimpan di Rust *core* sebagai operasi balik dari setiap transaksi (*Usulan*). Dampak perubahan terhadap *compile* tercantum pada Tabel 3.32.

**Tabel 3.32** Dampak perubahan dokumen

| Perubahan | Dampak |
| --- | --- |
| Mengubah rute *wire* tanpa mengubah *endpoint* | Hanya geometri, konektivitas tidak berubah |
| Menambah, menghapus, atau memindahkan *endpoint*, *junction*, pin, *tunnel*, *splitter*, atau *merger* | Konektivitas ditandai berubah dan memerlukan *compile* ulang |
| Mengubah lebar atau parameter yang memengaruhi pin | Konektivitas dan *lowering* ditandai berubah |
| Mengubah label | Tidak memengaruhi *compile* |
| Mengganti nama *tunnel* | *Tunnel* pada *scope* terkait diselesaikan ulang |

Tabel 3.32 menjadi dasar keputusan apakah tombol Simulate dapat memakai artefak yang sudah ada atau harus *compile* ulang. Perubahan geometri murni tidak mengubah *fingerprint* artefak.

#### 3.10.10 Memori, Periferal, dan Format Image

**Status.** *Usulan*. Format *image* ditetapkan sebelum M4, sedangkan periferal dan subset instruksi ditetapkan sebelum M8.

Isi ROM dimuat dari *file image* teks heksadesimal dengan satu kata per baris, lebar kata sesuai parameter komponen, dan baris kosong serta komentar diabaikan. Untuk demo CPU, masukan *keyboard* dimodelkan sebagai register masukan yang diperbarui oleh perintah SetInput pada batas siklus dan dibaca CPU melalui alamat memori (*memory-mapped*). Layar dimodelkan sebagai wilayah memori yang ditulis CPU dan ditampilkan UI dari `OutputBinding` pada *snapshot*. Subset RISC-V yang dipakai, peta memori, dan protokol tampilan dibatasi sebelum M8 untuk mengendalikan risiko R11, dengan kandidat awal RV32I tanpa ekstensi. Program contoh dibangun di luar DLF dengan *assembler* eksternal, karena DLF tidak menyediakan *assembler*.

#### 3.10.11 Algoritma Kunci

**Status.** *Usulan*. Detail penyelesaian *feedback* menunggu TBD-3.

1. **Konstruksi *net*.** Geometri dinormalisasi, segmen dipecah pada titik sambung, dan node dibuat untuk setiap pin, bit, dan *endpoint*. Node yang terhubung digabungkan memakai *disjoint set union* (DSU), lalu setiap kelompok diberi *Net ID* kanonis berdasarkan ID objek terkecil di dalamnya agar urutannya deterministik.
2. **Resolusi *driver*.** Pada empat nilai, beberapa *driver* diresolusi sesuai Tabel 2.1. Pada dua nilai, beberapa *driver* pada satu *net* menghasilkan diagnostik `NET-MULTI-DRIVER`.
3. **Graf ketergantungan kombinasional.** Simpulnya adalah operasi, dan sisinya mengalir dari operasi sumber ke operasi yang memakai keluarannya. Keluaran register dan masukan primer menjadi sumber pada level 0, sehingga elemen sekuensial memutus ketergantungan antarsiklus.
4. **Deteksi SCC.** Graf diperiksa dengan algoritma Tarjan. Kelompok dengan lebih dari satu simpul, atau satu simpul yang menunjuk dirinya sendiri, adalah wilayah *feedback*. Wilayah itu dikondensasi menjadi satu simpul sehingga graf menjadi DAG, lalu diselesaikan secara iteratif dengan batas iterasi yang ditetapkan pada TBD-3. Osilasi dilaporkan sebagai `DEP-SCC-OSCILLATION`.
5. ***Levelization*.** Level sebuah operasi adalah 1 ditambah level terbesar dari sumbernya. Operasi diurutkan per level, bila sama level diurutkan menurut ID stabil, dan `LevelRange` mencatat batas tiap level.
6. **Evaluasi.** Pada setiap siklus, operasi dijalankan berurutan per level. Karena tidak ada ketergantungan mundur antarlevel, setiap operasi sudah memiliki seluruh masukannya saat dijalankan.

Pengurutan berdasarkan ID stabil pada langkah 1 dan 5 menjamin bahwa dokumen yang sama selalu menghasilkan artefak yang sama (KNF-01).

#### 3.10.12 Format Fixture dan Golden Test

**Status.** *Usulan*, ditetapkan sebelum M1 dan dipakai mulai M3.

Setiap *fixture* berisi bidang berikut.

- ID dan deskripsi singkat tujuan uji.
- Dokumen rangkaian yang diuji.
- Versi profil semantik yang dipakai.
- Urutan masukan, berupa pasangan siklus, *binding*, dan nilai.
- Keluaran yang diharapkan, berupa pasangan siklus, sinyal atau *probe*, dan nilai, termasuk X dan Z bila relevan.
- *Seed* untuk pembangkitan acak pada *fuzzing*.

*Golden file* disimpan sebagai teks di samping *fixture*, dan pembandingan bersifat *bit-exact*. Uji yang gagal menyimpan *fixture*, urutan masukan, *seed*, serta keluaran yang diharapkan dan aktual. B0 memakai *fixture* yang sama tetapi tidak berbagi kode primitif dengan *runtime* yang diuji, sesuai catatan pada subbab 2.4.1.

#### 3.10.13 Keputusan yang Harus Ditutup per Milestone

Spesifikasi berstatus Usulan di atas dikumpulkan menurut milestone tempat ia harus ditutup, sehingga tidak ada yang tertinggal.

- **M0.** Daftar fungsi *facade* (3.10.7).
- **M1.** Format *file*, aturan ID, penanganan dependensi definisi komponen (3.10.1), dan format *fixture* (3.10.12).
- **M3.** Tabel kebenaran primitif dan perilaku *multiplexer* (TBD-1), daftar operasi IR dan pengodean empat nilai (TBD-2), serta algoritma SCC dan batas iterasi bila P1 dikerjakan (TBD-3).
- **M4.** *State* awal (TBD-4), prioritas *reset* dan *enable* (TBD-5), mode baca RAM dan perilaku tulis dengan alamat X (TBD-6), serta format *image* ROM.
- **M6.** Jenis perintah dan isi *snapshot* (3.10.6).
- **M8.** Subset RISC-V, peta memori, dan protokol tampilan (3.10.10).

## Referensi Sementara Bab III

Boehm, B. W. (1988). A spiral model of software development and enhancement. *Computer, 21*(5), 61–72. https://doi.org/10.1109/2.59

Brooke, J. (1996). SUS: A “quick and dirty” usability scale. In P. W. Jordan, B. Thomas, B. A. Weerdmeester, & I. L. McClelland (Eds.), *Usability evaluation in industry* (pp. 189–194). Taylor & Francis.

Czech Technical University in Prague. (n.d.). *Collection of digital design benchmarks*. https://ddd.fit.cvut.cz/www/prj/Benchmarks/index.php?page=download

Godot Engine. (n.d.). *Thread-safe APIs*. In *Godot documentation*. https://docs.godotengine.org/en/stable/tutorials/performance/thread\_safe\_apis.html

godot-rust. (n.d.). *Compatibility and stability*. In *The godot-rust book*. https://godot-rust.github.io/book/toolchain/compatibility.html

Larman, C., & Basili, V. R. (2003). Iterative and incremental developments: A brief history. *Computer, 36*(6), 47–56. https://doi.org/10.1109/MC.2003.1204375

NASA. (n.d.-a). *SWE-037 - Software milestones*. In *NASA software engineering handbook*. https://swehb.nasa.gov/pages/viewpage.action?navigatingVersions=true&pageId=209846280

NASA. (n.d.-b). *SWE-086 - Continuous risk management*. In *NASA software engineering handbook*. https://swehb.nasa.gov/spaces/SWEHBVB/pages/32604571/SWE-086%2B-%2BContinuous%2BRisk%2BManagement
