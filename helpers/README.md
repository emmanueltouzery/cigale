helper side-programs.
For instance, for redmine we need some translations, which we obtain
by parsing data from:
https://github.com/redmine/redmine/tree/master/config/locales

This is not part of the main cigale application though -- we just paste the output
from this helper application in cigale's redmine plugin.
