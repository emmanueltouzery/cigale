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

First tab, events:
![Main view picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/cigale-main.png)

Second tab, configured event sources:
![Event sources picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/cigale-event-sources.png)

## Installation

This is a rust and gtk application. I've tested it only on linux. The preferred way to install it is [through flathub](https://flathub.org/apps/details/com.github.emmanueltouzery.cigale), you can also of course build it from source.

## Use

You must configure event sources in the application so it displays your data.
Go in the second tab, then click the "New" button.

![Main view picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/first_run.png)

Once you have added event sources, you can return to the first tab to view the
data the application collected for you, one day at a time.
