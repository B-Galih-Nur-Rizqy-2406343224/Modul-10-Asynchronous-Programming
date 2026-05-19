# Module 10: Asynchronous Programming - Galih Nur Rizqy (2406343224)

## Experiment 1.2: Understanding how it works.

Ditambahkan satu baris `println!("Galih's Computer: hey hey!")` tepat setelah blok `spawner.spawn(...)` ditutup. Hasil yang muncul di konsol adalah `hey hey!` lebih dulu, baru `howdy!`, lalu setelah 2 detik `done!`, padahal secara urutan kode, `howdy!` ada di dalam spawn yang ditulis lebih atas.

Hal ini terjadi karena `spawner.spawn()` tidak langsung menjalankan future. Spawn hanya memasukkan task ke dalam antrian channel, dan future tersebut baru benar-benar dieksekusi ketika `executor.run()` dipanggil di bawahnya. Sementara itu, baris `println!` setelah spawn adalah kode sinkronus biasa yang langsung dijalankan thread utama.

Ini menunjukkan sifat *lazy* dari async Rust, future tidak berjalan sendiri tanpa executor yang meng-poll-nya. Pemisahan antara "mendaftarkan task" (spawn) dan "menjalankan task" (executor.run) adalah inti dari bagaimana async runtime bekerja.

![Hasil eksekusi Experiment 1.2](docs/img/1-2.png)

## Experiment 1.3: Multiple Spawn and removing drop

Pada eksperimen ini dilakukan dua hal: menambah jumlah spawn menjadi tiga, lalu mencoba menghapus `drop(spawner)`.

Dengan tiga spawn sekaligus, ketiga future dijadwalkan ke antrian sebelum `executor.run()` dipanggil. Saat executor mulai berjalan, ketiga task di-poll satu per satu secara bergantian. Setelah masing-masing mencapai `.await` pada `TimerFuture`, eksekusi task tersebut ditangguhkan dan executor beralih ke task berikutnya. Inilah yang membuat ketiga `howdy!` muncul hampir bersamaan di awal, lalu ketiga `done!` muncul setelah ~2 detik, urutannya tidak selalu 1-2-3 karena bergantung pada thread mana yang lebih dulu bangun dari sleep.

Ketika `drop(spawner)` dihapus, program tidak pernah keluar meskipun semua task sudah selesai. Penyebabnya adalah `executor.run()` berjalan dengan loop `while let Ok(task) = self.ready_queue.recv()`, loop ini baru berhenti kalau channel sudah ditutup. Channel baru ditutup ketika semua `Sender`-nya di-drop, dan satu-satunya Sender adalah `spawner`. Selama `spawner` masih hidup (belum didrop), `recv()` terus menunggu task baru yang tidak pernah datang, sehingga program hang selamanya.

Jadi peran masing-masing komponen: **spawner** mendaftarkan task ke antrian, **executor** mengeksekusi task-task tersebut, dan **drop(spawner)** adalah sinyal bahwa tidak ada lagi task yang akan masuk sehingga executor bisa berhenti dengan bersih.

![Hasil eksekusi Experiment 1.3 — dengan drop](docs/img/1-3a.png)

![Hasil eksekusi Experiment 1.3 — tanpa drop (program hang)](docs/img/1-3b.png)

## Experiment 2.1: Original code, and how it run

Kode broadcast chat ini diambil dari *Google Comprehensive Rust* dan dijalankan dengan satu server dan tiga client secara bersamaan. Cara menjalankannya: buka satu terminal untuk server dengan `cargo run --bin server`, lalu buka tiga terminal terpisah masing-masing untuk client dengan `cargo run --bin client`.

Ketika client terhubung, server langsung mengirim pesan sambutan "Welcome to chat! Type a message". Setelah itu, teks apapun yang diketik di salah satu client akan dikirim ke server sebagai WebSocket message, lalu server mem-broadcast pesan tersebut ke semua client yang sedang terhubung, termasuk pengirimnya sendiri.

Mekanisme ini bekerja dengan `tokio::broadcast::channel`: setiap client punya satu `Sender` (dikloning dari channel utama) dan satu `Receiver` (didapat dari `subscribe()`). Di dalam `handle_connection`, `tokio::select!` dipakai untuk secara bersamaan menunggu dua hal: pesan masuk dari client via WebSocket, dan pesan broadcast dari client lain via channel. Ini adalah pola konkurensi async yang efisien karena tidak membutuhkan thread terpisah per client.

WebSocket dipilih sebagai protokol transport karena sifatnya yang full-duplex, server dan client bisa saling mengirim kapan saja tanpa menunggu giliran, berbeda dengan HTTP biasa yang request-response. Ini sangat cocok untuk aplikasi chat real-time.

![Experiment 2.1 - Server dan tiga client](docs/img/2-1.png)
