if &cp | set nocp | endif
nnoremap <silent>  :CtrlP
map  :!urlview %
map ,h :s/\(.*\)/== \1 ==>\r<== \1 ==
map ,w :%s/\s$//g
map ,s :set spell spelllang=en_us
map ,r :grep -rn <cword> *
map ,p :cp
map ,k :wincmd k
map ,j :wincmd j
map ,i i r
map ,g :grep <cword> *[a-zA-Z0-9]
map ,e :w:cn
map ,d :cd %:h
map ,b :b#
map ,a :compiler ant|make -find build.xml
map ,S :set nospell
map ,m :set makeprg=make:make 
map ,D :cd %:h
map ,2 :only|copen|wincmd k
map ,1 :only
map ,x :confirm qa
map ,n :cn
vmap [% [%m'gv``
vmap ]% ]%m'gv``
map ]p :diffput
map ]g :diffget
vmap a% [%v]%
let s:cpo_save=&cpo
set cpo&vim
vmap gx <Plug>NetrwBrowseXVis
nmap gx <Plug>NetrwBrowseX
vnoremap <silent> <Plug>NetrwBrowseXVis :call netrw#BrowseXVis()
nnoremap <silent> <Plug>NetrwBrowseX :call netrw#BrowseX(expand((exists("g:netrw_gx")? g:netrw_gx : '<cfile>')),netrw#CheckIfRemote())
iabbr lenght length
let &cpo=s:cpo_save
unlet s:cpo_save
set paste
set background=dark
set backspace=indent,eol,start
set backupdir=/tmp
set cindent
set cinkeys=0{,0},0),:,!^F,o,O,e
set complete=.,w,b,u,t,i,kspell
set diffopt=filler,iwhite
set expandtab
set fileencodings=ucs-bom,utf-8,default,latin1
set nofsync
set helplang=en
set ignorecase
set matchpairs=(:),{:},[:],<:>
set path=**
set ruler
set runtimepath=~/.vim,/usr/share/vim/vimfiles,/usr/share/vim/vim74,/usr/share/vim/vimfiles/after,~/.vim/after,~/.vim/bundle/Vundle.vim,/after,~/Dropbox/vim
set shiftwidth=2
set showcmd
set showmatch
set softtabstop=2
set statusline=#%n\ 0X%B\ %l:%c/%L\ \ %f\ %y%m%r
set suffixes=.bak,~,.swp,.o,.info,.aux,.log,.dvi,.bbl,.blg,.brf,.cb,.ind,.idx,.ilg,.inx,.out,.toc,.png,.jpg
set suffixesadd=.java,.rb,,jsp,.js,.html,.xml,.hs
set swapsync=
set tabstop=2
set title
set titlestring=[\ %r%{CurDir()}\ ]\ %f
set updatecount=1
set nowritebackup
" vim: set ft=vim :
