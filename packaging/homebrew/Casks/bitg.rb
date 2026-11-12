cask "bitg" do
  version "0.2.0"
  sha256 "567c04804117ca8eb3993c19dabfe2198745591204cb15b908a76625c4db3f85"

  url "https://bitg.uz/download/bitg-#{version}-macos.pkg"
  name "bitg"
  desc "Uzbek input method for the new 2026 Latin alphabet"
  homepage "https://bitg.uz/"

  livecheck do
    url "https://github.com/dtwinuz/bitg"
    strategy :github_latest
  end

  depends_on macos: ">= :big_sur"

  pkg "bitg-#{version}-macos.pkg"

  uninstall quit:    "uz.bitg.inputmethod.bitg",
            pkgutil: "uz.bitg.inputmethod.pkg",
            delete:  "/Library/Input Methods/bitg.app"

  zap trash: "~/Library/Application Support/bitg"

  caveats <<~EOS
    Log out and back in, then choose bitg with Control + Space
    (System Settings > Keyboard > Input Sources > + > bitg).
  EOS
end
