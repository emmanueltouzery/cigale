# Cigale timesheet

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

![Main view picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/cigale-main.png)
![Event sources picture](https://raw.githubusercontent.com/wiki/emmanueltouzery/cigale/cigale-event-sources.png)

## Installation

This is a rust and gtk application. I've tested it only on linux. You can build it on your computer, I'm hoping to
have it soon available on flathub too.
