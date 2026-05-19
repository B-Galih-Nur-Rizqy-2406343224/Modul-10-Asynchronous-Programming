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
