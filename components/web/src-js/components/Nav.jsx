const details = document.getElementById("nav-mobile");

function closeMenu() {
  if (details) {
    details.open = false;
  }
}

if (details) {
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      closeMenu();
    }
  });

  document.addEventListener("click", (event) => {
    if (details.open && !details.contains(event.target)) {
      closeMenu();
    }
  });

  details.querySelectorAll("a").forEach((link) => {
    link.addEventListener("click", closeMenu);
  });
}
