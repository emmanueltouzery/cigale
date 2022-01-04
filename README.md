# Cigale timesheet

<a  href='https://flathub.org/apps/details/com.github.emmanueltouzery.cigale'><img width='180' align='right' alt='Download on Flathub' src='https://flathub.org/assets/badges/flathub-badge-en.png'/></a>

> "La Cigale, ayant chanté tout l'Été, se trouva fort dépourvue quand la bise fut venue."
> -- Jean de la Fontaine, The ant and the grasshopper

## Purpose

If you need to give a timesheet of your activities in the previous month for
your work, but you didn't collect
the information of what you were doing, when (like the grasshopper in the tale, who didn't plan for winter), then this program may help you.

It will look at traces of your past activity in your system. Here are the event sources that it will take into account:

- The emails you sent (mbox format, for instance Thunderbird)
- Ical sources (for instance Google calendar)
- Source control activity - Git
- Redmine bug activity
- Gitlab: issues activity, merge request comments and approvals
- Stack Exchange sites: your votes

First tab, events:
![Main view picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/cigale-main.png)

Second tab, configured event sources:
![Event sources picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/cigale-event-sources.png)

## Installation

This is a rust and gtk application.

### Linux

The preferred way to install it is [through flathub](https://flathub.org/apps/details/com.github.emmanueltouzery.cigale).

You can also build it from source. If you install all the system-related
dependencies and the rust toolchain, you should be able to simply run `cargo run --release`.
But on linux, you can also locally build and install a flatpak, without having
to install any dependencies yourself: `sh flatpak/build-and-install-flatpak.sh`.

### Mac OSX

You must [install the rust compiler](https://www.rust-lang.org/tools/install), then use homebrew to install a few dependencies:

    brew install gtk+3
    brew install adwaita-icon-theme
    brew install librsvg # may not be needed anymore, please report

Finally you can compile and run the application:

    cargo run --release

The binary will be in `target/release`, and is relocatable.
